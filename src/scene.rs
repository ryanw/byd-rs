use crate::pipelines::SimplePipeline;
use crate::MountContext;
use crate::{Actor, ActorID, Camera, DrawContext, Drawable, FreeCamera};
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use std::mem::size_of;
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};
use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
	1.0, 0.0, 0.0, 0.0,
	0.0, 1.0, 0.0, 0.0,
	0.0, 0.0, 0.5, 0.0,
	0.0, 0.0, 0.5, 1.0,
);

pub static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub struct ActorPod {
	pub id: ActorID,
	pub actor: Actor,
	uniform_offset: usize,
}

struct CameraUniform {
	pub view: Matrix4<f32>,
	pub projection: Matrix4<f32>,
	pub model: Matrix4<f32>,
}

impl CameraUniform {
	pub fn as_bytes(&self) -> &[u8] {
		unsafe {
			let ptr = self as *const CameraUniform as *const u8;
			let len = size_of::<Self>();
			std::slice::from_raw_parts(ptr, len)
		}
	}
}

pub struct RasterScene {
	camera: FreeCamera,
	actors: HashMap<ActorID, ActorPod>,
	pipeline: Option<SimplePipeline>,
	bind_group: Option<wgpu::BindGroup>,
	uniform_buffer: Option<wgpu::Buffer>,
}

impl RasterScene {
	pub fn new() -> Self {
		Self {
			camera: FreeCamera::new(),
			actors: HashMap::new(),
			bind_group: None,
			pipeline: None,
			uniform_buffer: None,
		}
	}

	pub fn build_pipeline(&mut self, ctx: &mut DrawContext) {
		let device = ctx.device();

		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Uniform Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: 1000 * uniform_alignment,
			mapped_at_creation: false,
		});

		let uniform_size = size_of::<CameraUniform>() as wgpu::BufferAddress;

		let pipeline = SimplePipeline::new(device);
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: pipeline.camera_bind_group_layout(),
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &uniform_buffer,
						offset: 0,
						size: wgpu::BufferSize::new(uniform_size),
					}),
				},
			],
			label: Some("uniform_bind_group"),
		});

		self.pipeline = Some(pipeline);
		self.bind_group = Some(bind_group);
		self.uniform_buffer = Some(uniform_buffer);
	}

	pub fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		if self.pipeline.is_none() {
			self.build_pipeline(ctx);
		}

		*self.camera.position_mut() = Point3::new(0.0, 0.0, -10.0);

		let mut camera_uniform = CameraUniform {
			view: self.camera.view(),
			projection: self.camera.projection(),
			model: Matrix4::identity(),
		};

		if let Some(pipeline) = self.pipeline.as_ref() {
			pipeline.apply(ctx.render_pass_mut());
		}

		let device = ctx.device();
		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		for ActorPod {
			actor,
			uniform_offset,
			..
		} in self.actors.values_mut()
		{
			camera_uniform.model = actor.transform.clone();
			let offset = (*uniform_offset * uniform_alignment as usize) as wgpu::DynamicOffset;
			if let Some(buffer) = self.uniform_buffer.as_ref() {
				// FIXME split camera and actor uniforms
				ctx.queue()
					.write_buffer(buffer, offset as _, camera_uniform.as_bytes());
			}

			if let Some(bind_group) = self.bind_group.as_ref() {
				ctx.render_pass_mut()
					.set_bind_group(0, bind_group, &[offset]);
			}
			actor.draw(ctx);
		}
	}

	pub fn add(&mut self, mut actor: Actor) {
		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		let uniform_offset = self.actors.len();
		let mut ctx = MountContext { actor_id: id };
		actor.mount(&mut ctx);
		self.actors.insert(
			id,
			ActorPod {
				id,
				actor,
				uniform_offset,
			},
		);
	}

	pub fn remove(&mut self, id: ActorID) {
		if let Some(mut pod) = self.actors.remove(&id) {
			let mut ctx = MountContext { actor_id: id };
			pod.actor.unmount(&mut ctx);
		}
	}

	/// Get a reference to the scene's camera.
	pub fn camera(&self) -> &FreeCamera {
		&self.camera
	}

	/// Get a mutable reference to the scene's camera.
	pub fn camera_mut(&mut self) -> &mut FreeCamera {
		&mut self.camera
	}

	/// Get a reference to the scene's actors.
	pub fn actors(&self) -> &HashMap<ActorID, ActorPod> {
		&self.actors
	}

	/// Get a mutable reference to the scene's actors.
	pub fn actors_mut(&mut self) -> &mut HashMap<ActorID, ActorPod> {
		&mut self.actors
	}
}
