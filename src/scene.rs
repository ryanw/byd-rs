use crate::{
	pipelines::{ActorUniform, CameraUniform, SimplePipeline, ACTOR_BINDING, CAMERA_BINDING},
	BasicMaterial, Camera, Material, MountContext, Pipeline, RenderContext,
};
use cgmath::{Matrix4, SquareMatrix, Vector4};
use downcast_rs::{impl_downcast, Downcast};
use std::{
	collections::{HashMap, HashSet},
	mem::size_of,
	sync::atomic::{AtomicUsize, Ordering},
};

const MAX_ACTORS: u64 = 2048;

pub type ObjectID = usize;
pub static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub trait SceneObject: Downcast {
	fn render<'a>(&'a self, _ctx: &mut RenderContext<'a>) {}
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

pub struct SceneUniforms {
	pipeline: SimplePipeline,
	bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	actor_buffer: wgpu::Buffer,
}

impl SceneUniforms {
	pub fn new(device: &wgpu::Device) -> Self {
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

		let camera_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("QuadPipeline Bind Group"),
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

pub struct Scene {
	objects: HashMap<ObjectID, Box<dyn SceneObject>>,
	added: HashSet<ObjectID>,
	removed: HashSet<ObjectID>,
	uniforms: Option<SceneUniforms>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			objects: HashMap::new(),
			added: HashSet::new(),
			removed: HashSet::new(),
			uniforms: None,
		}
	}

	pub fn render<'a>(&'a mut self, ctx: &mut RenderContext<'a>) {
		let uniforms = self
			.uniforms
			.get_or_insert_with(|| SceneUniforms::new(ctx.device));

		let mut mount_ctx = MountContext { device: ctx.device };

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

		for (id, object) in &mut self.objects {
			if let Some(material) = object
				.material()
				.and_then(|o| o.downcast_ref::<BasicMaterial>())
			{
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
