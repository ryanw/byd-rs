use byd::{
	Actor, App, AttachContext, DrawContext, Event, Material, Mesh, PipelineID, RasterScene,
	SimpleVertex, UpdateContext, Vertex, Window,
};

struct SimplePipeline {
	render_pipeline: wgpu::RenderPipeline,
}

impl SimplePipeline {
	fn new(device: &wgpu::Device) -> Self {
		log::debug!("Creating vertex shader");
		let vs_module =
			device.create_shader_module(&wgpu::include_spirv!("../shaders/simple.vert.spv"));
		log::debug!("Creating fragment shader");
		let fs_module =
			device.create_shader_module(&wgpu::include_spirv!("../shaders/simple.frag.spv"));

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		log::debug!("Creating pipeline");
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Cube Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &vs_module,
				entry_point: "main",
				buffers: &[SimpleVertex::buffer_layout()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &fs_module,
				entry_point: "main",
				targets: &[wgpu::ColorTargetState {
					format: wgpu::TextureFormat::Bgra8UnormSrgb, // FIXME ctx.swapchain_format(),
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrite::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				//cull_mode: Some(wgpu::Face::Back),
				cull_mode: None,
				clamp_depth: false,
				conservative: false,
				polygon_mode: wgpu::PolygonMode::Fill,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
		});

		Self {
			render_pipeline: pipeline,
		}
	}

	pub fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_pipeline(&self.render_pipeline);
	}
}

struct CubeApp {
	scene: Option<RasterScene>,
	pipeline: Option<SimplePipeline>,
}

impl CubeApp {
	pub fn new() -> Self {
		Self {
			scene: None,
			pipeline: None,
		}
	}
}

impl App for CubeApp {
	fn attach(&mut self, ctx: &mut AttachContext) {
		let device = ctx.device();
		let mut scene = RasterScene::new(device);
		let pipeline = SimplePipeline::new(device);
		let cube = Actor {
			geometry: Box::new(Mesh::cube(0.1)),
			material: Material::default(),
		};

		scene.add(cube);
		self.scene = Some(scene);
		self.pipeline = Some(pipeline);
	}

	fn draw<'a>(&'a mut self, ctx: &mut DrawContext, render_pass: &mut wgpu::RenderPass<'a>) {
		if let Some(scene) = &mut self.scene {
			if let Some(pipeline) = self.pipeline.as_ref() {
				pipeline.apply(render_pass);
				scene.draw(ctx, render_pass);
			}
		}
	}

	fn update(&mut self, ctx: &mut UpdateContext) {
		let dt = ctx.dt().as_secs_f32();
	}

	fn event(&mut self, event: &Event) {}
}

fn main() {
	env_logger::init();
	Window::new().run(CubeApp::new());
}
