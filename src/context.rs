use crate::window::State;
use crate::{ActorID, AsUniformValue, PipelineID, UniformMap};
use std::time::Duration;

pub struct AttachContext<'a> {
	pub(crate) state: &'a mut State,
}

pub struct DrawContext<'a> {
	pub(crate) dt: Duration,
	pub(crate) uniforms: UniformMap,
	pub(crate) viewport_size: (f32, f32),
	pub(crate) state: &'a mut State,
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
		self.state.sc_desc.format
	}

	pub fn add_pipeline(&mut self, pipeline: wgpu::RenderPipeline) -> PipelineID {
		self.state.add_pipeline(pipeline)
	}
}

impl<'a> DrawContext<'a> {
	pub fn new(state: &'a mut State, render_pass: wgpu::RenderPass<'a>) -> Self {
		Self {
			dt: Duration::default(),
			uniforms: UniformMap::new(),
			viewport_size: (0.0, 0.0),
			state,
			render_pass,
		}
	}

	pub fn render_pass(&self) -> &wgpu::RenderPass<'a> {
		&self.render_pass
	}

	pub fn render_pass_mut(&mut self) -> &mut wgpu::RenderPass<'a> {
		&mut self.render_pass
	}

	pub fn device(&self) -> &wgpu::Device {
		&self.state.device
	}

	pub fn pipeline(&self, id: PipelineID) -> Option<&wgpu::RenderPipeline> {
		self.state.pipelines.get(&id)
	}

	pub fn clear_color(&mut self, r: f32, g: f32, b: f32, a: f32) {
		todo!();
	}

	pub fn use_pipeline(&mut self, id: PipelineID) {
		log::debug!("Changing render pipeline to : {}", id);
		self.state.use_pipeline(id);
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
