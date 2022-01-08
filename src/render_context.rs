use crate::Camera;

pub struct RenderContext<'a> {
	pub device: &'a wgpu::Device,
	pub queue: &'a mut wgpu::Queue,
	pub render_pass: wgpu::RenderPass<'a>,
	pub camera: &'a dyn Camera,
}
