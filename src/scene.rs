use crate::pipelines::SimplePipeline;
use crate::MountContext;
use crate::{Actor, ActorID, Camera, DrawContext, Drawable, FreeCamera};
use cgmath::{Point3, Transform};
use std::mem::size_of;
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};
use wgpu::util::DeviceExt;

pub static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub struct RasterScene {
	camera: FreeCamera,
	actors: HashMap<ActorID, Actor>,
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

		let uniform_contents = unsafe {
			let color = [0.0f32, 0.0, 1.0, 1.0];
			let len = color.len() * size_of::<f32>();
			let ptr = color.as_ptr() as *const u8;
			std::slice::from_raw_parts(ptr, len)
		};
		let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Uniform Buffer"),
			contents: uniform_contents,
			usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
		});

		let pipeline = SimplePipeline::new(device);
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: pipeline.bind_group_layout(),
			entries: &[wgpu::BindGroupEntry {
				binding: 0,
				resource: uniform_buffer.as_entire_binding(),
			}],
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

		let view = self.camera.view();
		if let Some(buffer) = self.uniform_buffer.as_ref() {
			let uniform_contents = unsafe {
				let color = [1.0f32, 0.0, 1.0, 1.0];
				let len = color.len() * size_of::<f32>();
				let ptr = color.as_ptr() as *const u8;
				std::slice::from_raw_parts(ptr, len)
			};
			ctx.queue().write_buffer(buffer, 0, uniform_contents);
		}

		if let Some(pipeline) = self.pipeline.as_ref() {
			pipeline.apply(ctx.render_pass_mut());
		}

		if let Some(bind_group) = self.bind_group.as_ref() {
			ctx.render_pass_mut().set_bind_group(0, bind_group, &[]);
		}

		for (_id, actor) in &mut self.actors {
			actor.draw(ctx)
		}
	}

	pub fn add(&mut self, mut actor: Actor) {
		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		let mut ctx = MountContext { actor_id: id };
		actor.mount(&mut ctx);
		self.actors.insert(id, actor);
	}

	pub fn remove(&mut self, id: ActorID) {
		if let Some(mut actor) = self.actors.remove(&id) {
			let mut ctx = MountContext { actor_id: id };
			actor.unmount(&mut ctx);
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
	pub fn actors(&self) -> &HashMap<ActorID, Actor> {
		&self.actors
	}

	/// Get a mutable reference to the scene's actors.
	pub fn actors_mut(&mut self) -> &mut HashMap<ActorID, Actor> {
		&mut self.actors
	}
}
