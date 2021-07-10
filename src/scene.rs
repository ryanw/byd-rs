use crate::MountContext;
use crate::{Actor, ActorID, Camera, DrawContext, Drawable, FreeCamera};
use cgmath::{Point3, Transform};
use std::{
	collections::HashMap,
	sync::atomic::{AtomicUsize, Ordering},
};

pub static NEXT_ID: AtomicUsize = AtomicUsize::new(1);

pub struct RasterScene {
	camera: FreeCamera,
	actors: HashMap<ActorID, Actor>,
}

impl RasterScene {
	pub fn new(device: &wgpu::Device) -> Self {
		Self {
			camera: FreeCamera::new(),
			actors: HashMap::new(),
		}
	}

	pub fn draw<'a>(&'a mut self, ctx: &mut DrawContext, render_pass: &mut wgpu::RenderPass<'a>) {
		let view = self.camera.view();
		let u_camera: [f32; 3] = view.transform_point(Point3::new(0.0, 0.0, 0.0)).into();
		let u_look_at: [f32; 3] = view.transform_point(Point3::new(0.0, 0.0, 1.0)).into();
		ctx.insert_uniform("uCamera", u_camera);
		ctx.insert_uniform("uLookAt", u_look_at);
		ctx.insert_uniform("uFogColor", [0.4, 0.3, 0.2]);
		for (_id, actor) in &mut self.actors {
			actor.draw(ctx, render_pass)
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
