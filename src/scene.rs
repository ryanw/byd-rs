use crate::pipelines::{ActorUniform, CameraUniform, SimplePipeline, Uniform};
use crate::{Actor, ActorID, Camera, DrawContext, Drawable, FreeCamera, RenderContext};
use crate::{Mesh, MountContext};
use cgmath::{Matrix4, Point3};
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

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

pub struct Scene {
	camera: FreeCamera,
	actors: HashMap<ActorID, ActorPod>,
	pipeline: Option<SimplePipeline>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			camera: FreeCamera::new(),
			actors: HashMap::new(),
			pipeline: None,
		}
	}

	pub fn build_pipeline(&mut self, ctx: &mut DrawContext) {
		self.pipeline = Some(SimplePipeline::new(ctx.device()));
	}

	pub fn render<'a>(&'a self, ctx: &mut RenderContext<'a>) {}

	pub fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		if self.pipeline.is_none() {
			self.build_pipeline(ctx);
		}

		*self.camera.position_mut() = Point3::new(0.0, 0.0, -10.0);

		if let Some(pipeline) = self.pipeline.as_ref() {
			pipeline.apply(ctx);

			// Update camera position
			pipeline.set_camera(
				ctx,
				&CameraUniform {
					view: self.camera.view(),
					projection: self.camera.projection(),
				},
			);

			// Update actor positions
			for pod in self.actors.values_mut() {
				pipeline.set_actor(
					ctx,
					pod.uniform_offset,
					&ActorUniform {
						model: pod.actor.transform.clone(),
					},
				);
				pipeline.bind_actor(ctx, pod.uniform_offset);
				pod.actor.draw(ctx);
			}
		}
	}

	pub fn add(&mut self, mesh: Mesh) -> usize {
		0
	}

	pub fn get_mut(&mut self, id: usize) -> Option<&mut Mesh> {
		None
	}

	pub fn old_add(&mut self, mut actor: Actor) {
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
