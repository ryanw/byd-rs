use crate::Camera;

pub struct RenderContext<'a> {
	pub device: &'a wgpu::Device,
	pub queue: &'a wgpu::Queue,
	pub render_pass: &'a wgpu::RenderPass<'a>,
	pub camera: &'a dyn Camera,
}
