use crate::{AttachContext, DrawContext, Event, UpdateContext};

pub trait App {
	fn attach(&mut self, _ctx: &mut AttachContext) {}
	fn draw<'a>(&'a mut self, ctx: &mut DrawContext, render_pass: &mut wgpu::RenderPass<'a>);
	fn update(&mut self, _ctx: &mut UpdateContext) {}
	fn event(&mut self, _event: &Event) {}
}
