use crate::{DrawContext, Drawable, Material, MountContext};

pub struct Actor {
	pub geometry: Box<dyn Drawable>,
	pub material: Material,
}

impl Drawable for Actor {
	fn draw<'a>(&'a mut self, ctx: &mut DrawContext, render_pass: &mut wgpu::RenderPass<'a>) {
		self.geometry.draw(ctx, render_pass);
	}

	fn mount(&mut self, ctx: &mut MountContext) {
		self.geometry.mount(ctx);
	}

	fn unmount(&mut self, ctx: &mut MountContext) {
		self.geometry.unmount(ctx);
	}
}