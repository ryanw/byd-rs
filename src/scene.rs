use crate::{
	pipelines::{
		ActorUniform, CameraUniform, LinePipeline, SimplePipeline, ACTOR_BINDING, CAMERA_BINDING,
		SAMPLER_BINDING, TEXTURE_BINDING, TEXTURE_ENABLED_BINDING,
	},
	BasicMaterial, Camera, Color, LineMaterial, MountContext, Pipeline, RenderContext, SceneObject,
	Texture, TextureBuffer, TextureMaterial,
};
use cgmath::Vector4;
use std::{
	collections::{HashMap, HashSet},
	mem::size_of,
	sync::atomic::{AtomicUsize, Ordering},
};

const MAX_OBJECTS: u64 = 2048;

pub type ObjectID = usize;
pub type TextureID = usize;
pub static NEXT_OBJECT_ID: AtomicUsize = AtomicUsize::new(1);
pub static NEXT_TEXTURE_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Scene {
	objects: HashMap<ObjectID, Box<dyn SceneObject>>,
	textures: HashMap<TextureID, Texture>,
	uniforms: Option<SceneUniforms>,
	debug_uniforms: Option<DebugUniforms>,

	added: HashSet<ObjectID>,
	removed: HashSet<ObjectID>,
	added_textures: HashSet<TextureID>,
	removed_textures: HashSet<TextureID>,
}

impl Scene {
	pub fn new() -> Self {
		let mut scene = Self {
			objects: HashMap::new(),
			textures: HashMap::new(),
			uniforms: None,
			debug_uniforms: None,
			added: HashSet::new(),
			removed: HashSet::new(),
			added_textures: HashSet::new(),
			removed_textures: HashSet::new(),
		};

		// Add a default texture
		let id = scene.add_texture(
			Texture::from_image_bytes(include_bytes!("../assets/pixel.png"))
				.expect("Failed to load default texture"),
		);
		assert!(id == 0);

		scene
	}

	pub fn add_texture(&mut self, texture: Texture) -> TextureID {
		let id = NEXT_TEXTURE_ID.fetch_add(1, Ordering::Relaxed);
		self.textures.insert(id, texture);
		self.added_textures.insert(id);
		id
	}

	pub fn process_texture_queue(&mut self, device: &wgpu::Device, queue: &mut wgpu::Queue) {
		if let Some(uniforms) = self.uniforms.as_mut() {
			// Add flagged objects
			for id in self.added_textures.drain() {
				if let Some(texture) = self.textures.get_mut(&id) {
					texture.allocate(device, "Some Texture");
					texture.upload(queue);
					uniforms.add_texture(id, device, texture.buffer().unwrap());
				}
			}

			// Remove flagged objects
			for id in self.removed_textures.drain() {
				self.textures.remove(&id);
			}
		}
	}

	pub fn render<'a>(&'a mut self, ctx: &mut RenderContext<'a>) {
		let uniforms = self
			.uniforms
			.get_or_insert_with(|| SceneUniforms::new(ctx.device, ctx.queue));
		let debug_uniforms = self
			.debug_uniforms
			.get_or_insert_with(|| DebugUniforms::new(ctx.device));

		let mut mount_ctx = MountContext { device: ctx.device };

		// Default image bind group hasn't been created yet.
		if uniforms.texture_bind_groups.len() == 0 {
			return;
		}

		// Add flagged objects
		for id in self.added.drain() {
			if let Some(object) = self.objects.get_mut(&id) {
				object.mount(&mut mount_ctx);
			}
		}

		// Remove flagged objects
		for id in self.removed.drain() {
			if let Some(mut object) = self.objects.remove(&id) {
				object.unmount(&mut mount_ctx);
			}
		}

		// Update camera position
		uniforms.set_camera(ctx, ctx.camera);
		debug_uniforms.set_camera(ctx, ctx.camera);

		for (id, object) in &mut self.objects {
			let material = object.material();
			if let Some(material) = material.downcast_ref::<BasicMaterial>() {
				// Update object position
				uniforms.set_actor(
					ctx,
					*id as _,
					ActorUniform {
						color: material.color,
						model: object.transform(),
					},
				);

				// Render object
				uniforms.bind_actor(ctx, *id as _);
				uniforms.bind_texture(ctx, 0);
				object.render(ctx);
			} else if let Some(material) = material.downcast_ref::<TextureMaterial>() {
				// Update object position
				uniforms.set_actor(
					ctx,
					*id as _,
					ActorUniform {
						color: Vector4::new(0.0, 0.0, 0.0, 1.0),
						model: object.transform(),
					},
				);

				// Render object
				uniforms.bind_actor(ctx, *id as _);
				uniforms.bind_texture(ctx, material.texture_id);
				object.render(ctx);
			} else if let Some(_material) = material.downcast_ref::<LineMaterial>() {
				// Update object position
				debug_uniforms.set_actor(
					ctx,
					*id as _,
					ActorUniform {
						color: Color::new(1.0, 0.0, 1.0, 1.0),
						model: object.transform(),
					},
				);

				// Render object
				debug_uniforms.bind_actor(ctx, *id as _);
				object.render(ctx);
			}
		}
	}

	pub fn add<O>(&mut self, object: O) -> ObjectID
	where
		O: 'static + SceneObject,
	{
		let id = NEXT_OBJECT_ID.fetch_add(1, Ordering::Relaxed);
		self.objects.insert(id, Box::new(object));
		self.added.insert(id);
		id
	}

	pub fn get(&self, id: ObjectID) -> Option<&Box<dyn SceneObject>> {
		self.objects.get(&id)
	}

	pub fn get_mut(&mut self, id: ObjectID) -> Option<&mut Box<dyn SceneObject>> {
		self.objects.get_mut(&id)
	}

	pub fn remove(&mut self, id: ObjectID) {
		if self.objects.contains_key(&id) {
			self.removed.insert(id);
		}
	}

	pub fn with_object<O, F>(&self, id: ObjectID, handler: F)
	where
		O: SceneObject,
		F: FnOnce(&O),
	{
		if let Some(obj) = self.get(id).and_then(|obj| obj.downcast_ref::<O>()) {
			handler(obj);
		}
	}

	pub fn with_object_mut<O, F>(&mut self, id: ObjectID, handler: F)
	where
		O: SceneObject,
		F: FnOnce(&mut O),
	{
		if let Some(obj) = self.get_mut(id).and_then(|obj| obj.downcast_mut::<O>()) {
			handler(obj);
		}
	}
}

pub struct DebugUniforms {
	pipeline: LinePipeline,
	bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	actor_buffer: wgpu::Buffer,
}

impl DebugUniforms {
	pub fn new(device: &wgpu::Device) -> Self {
		log::debug!("Building Debug Uniforms");
		let pipeline = LinePipeline::new(device);

		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: uniform_alignment,
			mapped_at_creation: false,
		});

		let actor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Actor Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: MAX_OBJECTS * uniform_alignment,
			mapped_at_creation: false,
		});

		let camera_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("LinePipeline Bind Group"),
			layout: pipeline.bind_group_layout(),
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: CAMERA_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						size: wgpu::BufferSize::new(camera_size),
						offset: 0,
					}),
				},
				// Actors
				wgpu::BindGroupEntry {
					binding: ACTOR_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &actor_buffer,
						size: wgpu::BufferSize::new(actor_size),
						offset: 0,
					}),
				},
			],
		});

		Self {
			pipeline,
			bind_group,
			camera_buffer,
			actor_buffer,
		}
	}

	fn set_camera(&self, ctx: &mut RenderContext, camera: &dyn Camera) {
		let contents = CameraUniform {
			view: camera.view(),
			projection: camera.projection(),
		};
		ctx.queue
			.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[contents]));
	}

	fn set_actor(&self, ctx: &mut RenderContext, index: u64, contents: ActorUniform) {
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		ctx.queue.write_buffer(
			&self.actor_buffer,
			offset as _,
			bytemuck::cast_slice(&[contents]),
		);
	}

	fn bind_actor<'a>(&'a self, ctx: &mut RenderContext<'a>, index: u64) {
		let render_pass = &mut ctx.render_pass;
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		self.pipeline.apply(render_pass);
		render_pass.set_bind_group(0, &self.bind_group, &[offset]);
	}
}

pub struct SceneUniforms {
	pipeline: SimplePipeline,
	bind_group: wgpu::BindGroup,
	texture_bind_groups: HashMap<TextureID, wgpu::BindGroup>,
	camera_buffer: wgpu::Buffer,
	actor_buffer: wgpu::Buffer,
	enabled_buffer: wgpu::Buffer,
}

impl SceneUniforms {
	pub fn new(device: &wgpu::Device, queue: &mut wgpu::Queue) -> Self {
		log::debug!("Building Scene Uniforms");
		let pipeline = SimplePipeline::new(device);

		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: uniform_alignment,
			mapped_at_creation: false,
		});

		let actor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Actor Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: MAX_OBJECTS * uniform_alignment,
			mapped_at_creation: false,
		});

		let enabled_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Texture Enabled Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: 2 * uniform_alignment,
			mapped_at_creation: false,
		});

		queue.write_buffer(&enabled_buffer, 0, bytemuck::cast_slice(&[0]));
		queue.write_buffer(
			&enabled_buffer,
			uniform_alignment,
			bytemuck::cast_slice(&[1]),
		);

		let camera_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("SimplePipeline Bind Group"),
			layout: pipeline.bind_group_layout(),
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: CAMERA_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						size: wgpu::BufferSize::new(camera_size),
						offset: 0,
					}),
				},
				// Actors
				wgpu::BindGroupEntry {
					binding: ACTOR_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &actor_buffer,
						size: wgpu::BufferSize::new(actor_size),
						offset: 0,
					}),
				},
			],
		});

		Self {
			pipeline,
			bind_group,
			texture_bind_groups: HashMap::new(),
			camera_buffer,
			actor_buffer,
			enabled_buffer,
		}
	}

	fn set_camera(&self, ctx: &mut RenderContext, camera: &dyn Camera) {
		let contents = CameraUniform {
			view: camera.view(),
			projection: camera.projection(),
		};
		ctx.queue
			.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[contents]));
	}

	fn set_actor(&self, ctx: &mut RenderContext, index: u64, contents: ActorUniform) {
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		ctx.queue.write_buffer(
			&self.actor_buffer,
			offset as _,
			bytemuck::cast_slice(&[contents]),
		);
	}

	fn bind_actor<'a>(&'a self, ctx: &mut RenderContext<'a>, index: u64) {
		let render_pass = &mut ctx.render_pass;
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		self.pipeline.apply(render_pass);
		render_pass.set_bind_group(0, &self.bind_group, &[offset]);
	}

	fn bind_texture<'a>(&'a self, ctx: &mut RenderContext<'a>, id: TextureID) {
		if let Some(texture) = self.texture_bind_groups.get(&id) {
			let uniform_alignment =
				ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
			let is_enabled_offset =
				(if id == 0 { 0 } else { uniform_alignment }) as wgpu::DynamicOffset;

			ctx.render_pass
				.set_bind_group(1, texture, &[is_enabled_offset]);
		}
	}

	fn add_texture(&mut self, id: TextureID, device: &wgpu::Device, texture: &TextureBuffer) {
		log::debug!("Creating BindGroup for texture {}", id);
		self.texture_bind_groups.insert(
			id,
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("SimplePipeline Texture Bind Group"),
				layout: self.pipeline.texture_bind_group_layout(),
				entries: &[
					wgpu::BindGroupEntry {
						binding: TEXTURE_ENABLED_BINDING,
						resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
							buffer: &self.enabled_buffer,
							size: wgpu::BufferSize::new(size_of::<u32>() as _),
							offset: 0,
						}),
					},
					wgpu::BindGroupEntry {
						binding: TEXTURE_BINDING,
						resource: wgpu::BindingResource::TextureView(&texture.view),
					},
					wgpu::BindGroupEntry {
						binding: SAMPLER_BINDING,
						resource: wgpu::BindingResource::Sampler(&texture.sampler),
					},
				],
			}),
		);
	}
}
