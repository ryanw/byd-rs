use crate::{
	pipelines::{
		ActorUniform, CameraUniform, LinePipeline, SimplePipeline, ACTOR_BINDING, CAMERA_BINDING,
		SAMPLER_BINDING, TEXTURE_BINDING, TEXTURE_ENABLED_BINDING,
	},
	BasicMaterial, Camera, Color, LineMaterial, Material, MountContext, Pipeline, RenderContext,
	Texture, TextureMaterial,
};
use cgmath::{Matrix4, SquareMatrix, Vector4};
use downcast_rs::{impl_downcast, Downcast};
use image::{DynamicImage, ImageError};
use std::{
	collections::{HashMap, HashSet, VecDeque},
	mem::size_of,
	sync::atomic::{AtomicUsize, Ordering},
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

const MAX_ACTORS: u64 = 2048;

pub type ObjectID = usize;
pub static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub trait SceneObject: Downcast {
	fn render<'a>(&'a mut self, _ctx: &mut RenderContext<'a>) {}
	fn mount(&mut self, _ctx: &mut MountContext) {}
	fn unmount(&mut self, _ctx: &mut MountContext) {}
	fn transform(&self) -> Matrix4<f32> {
		Matrix4::identity()
	}
	fn material(&self) -> Option<&dyn Material> {
		None
	}
}
impl_downcast!(SceneObject);

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
			size: MAX_ACTORS * uniform_alignment,
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
	texture_bind_groups: Vec<wgpu::BindGroup>,
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
			size: MAX_ACTORS * uniform_alignment,
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
			texture_bind_groups: vec![],
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

	fn bind_texture<'a>(&'a self, ctx: &mut RenderContext<'a>, index: usize) {
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (if index == 0 { 0 } else { 1 } * uniform_alignment) as wgpu::DynamicOffset;

		ctx.render_pass
			.set_bind_group(1, &self.texture_bind_groups[index], &[offset]);
	}

	fn add_texture(&mut self, device: &wgpu::Device, texture: &Texture) -> usize {
		let id = self.texture_bind_groups.len() + 1;

		log::debug!("Creating BindGroup for texture {}", id);
		self.texture_bind_groups
			.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
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
			}));

		id
	}
}

pub struct Scene {
	objects: HashMap<ObjectID, Box<dyn SceneObject>>,
	uniforms: Option<SceneUniforms>,
	debug_uniforms: Option<DebugUniforms>,

	added: HashSet<ObjectID>,
	removed: HashSet<ObjectID>,
	texture_queue: VecDeque<DynamicImage>,
}

impl Scene {
	pub fn new() -> Self {
		let mut scene = Self {
			objects: HashMap::new(),
			uniforms: None,
			debug_uniforms: None,
			added: HashSet::new(),
			removed: HashSet::new(),
			texture_queue: VecDeque::new(),
		};

		// Add a default texture
		scene
			.load_texture_from_bytes(include_bytes!("../assets/pixel.png"))
			.unwrap();

		scene
	}

	pub fn load_texture_from_bytes(&mut self, bytes: &[u8]) -> Result<(), ImageError> {
		self.texture_queue
			.push_back(image::load_from_memory(bytes)?);

		log::debug!("Added texture to load queue");
		Ok(())
	}

	pub fn process_texture_queue(&mut self, device: &wgpu::Device, queue: &mut wgpu::Queue) {
		if let Some(uniforms) = self.uniforms.as_mut() {
			if self.texture_queue.len() > 0 {
				log::debug!(
					"Processing texture queue: {} items",
					self.texture_queue.len()
				);
			}
			for image in self.texture_queue.drain(..) {
				let rgba = image.as_rgba8().expect("Image isn't RGBA8");
				let texture = Texture::new(device, rgba.width(), rgba.height(), "Some Texture");
				texture.write(queue, rgba);
				uniforms.add_texture(device, &texture);
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

		let default_material = BasicMaterial::new(Color::new(1.0, 0.0, 0.0, 1.0));
		for (id, object) in &mut self.objects {
			let material = object.material().unwrap_or(&default_material);
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
		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
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
