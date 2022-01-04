use crate::{DrawContext, Drawable, Material, MountContext};
use cgmath::Matrix4;

pub struct Actor {
	pub geometry: Box<dyn Drawable>,
	pub material: Material,
	pub transform: Matrix4<f32>,
}

impl Drawable for Actor {
	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		self.geometry.draw(ctx);
	}

	fn mount(&mut self, ctx: &mut MountContext) {
		self.geometry.mount(ctx);
	}

	fn unmount(&mut self, ctx: &mut MountContext) {
		self.geometry.unmount(ctx);
	}
}
