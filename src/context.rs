use crate::State;
use crate::{ActorID, AsUniformValue, PipelineID, UniformMap};
use std::time::Duration;

pub struct AttachContext<'a> {
	pub(crate) state: &'a mut State,
}

pub struct DrawContext<'a> {
	pub(crate) dt: Duration,
	pub(crate) uniforms: UniformMap,
	pub(crate) viewport_size: (f32, f32),
	pub(crate) device: &'a wgpu::Device,
	pub(crate) queue: &'a mut wgpu::Queue,
	render_pass: wgpu::RenderPass<'a>,
}

#[derive(Clone)]
pub struct MountContext {
	pub(crate) actor_id: ActorID,
}

#[derive(Clone)]
pub struct UpdateContext {
	pub(crate) dt: Duration,
}

impl<'a> AttachContext<'a> {
	pub fn new(state: &'a mut State) -> Self {
		Self { state }
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.state.device
	}

	pub fn swapchain_format(&self) -> wgpu::TextureFormat {
		if let Some(sc_desc) = self.state.sc_desc.as_ref() {
			sc_desc.format
		} else {
			wgpu::TextureFormat::Bgra8UnormSrgb
		}
	}
}

impl<'a> DrawContext<'a> {
	pub fn new(
		device: &'a wgpu::Device,
		queue: &'a mut wgpu::Queue,
		render_pass: wgpu::RenderPass<'a>,
	) -> Self {
		Self {
			dt: Duration::default(),
			uniforms: UniformMap::new(),
			viewport_size: (0.0, 0.0),
			device,
			queue,
			render_pass,
		}
	}

	pub fn render_pass(&self) -> &wgpu::RenderPass<'a> {
		&self.render_pass
	}

	pub fn render_pass_mut(&mut self) -> &mut wgpu::RenderPass<'a> {
		&mut self.render_pass
	}

	pub fn queue(&self) -> &wgpu::Queue {
		self.queue
	}

	pub fn queue_mut(&mut self) -> &mut wgpu::Queue {
		self.queue
	}

	pub fn device(&self) -> &wgpu::Device {
		self.device
	}

	pub fn insert_uniform(&mut self, name: &str, value: impl AsUniformValue) {
		self.uniforms.insert(name, value);
	}

	/// Get a reference to the draw context's uniforms.
	pub fn uniforms(&self) -> &UniformMap {
		&self.uniforms
	}

	/// Get a mutable reference to the draw context's uniforms.
	pub fn uniforms_mut(&mut self) -> &mut UniformMap {
		&mut self.uniforms
	}

	/// Get a reference to the draw context's dt.
	pub fn dt(&self) -> Duration {
		self.dt
	}

	/// Get a reference to the draw context's viewport size.
	pub fn viewport_size(&self) -> &(f32, f32) {
		&self.viewport_size
	}
}

impl MountContext {
	// TODO
}

impl UpdateContext {
	/// Get a reference to the update context's dt.
	pub fn dt(&self) -> Duration {
		self.dt
	}
}
