use byd::{
	Actor, App, AttachContext, DrawContext, Event, Material, Mesh, RasterScene,
	SimpleVertex, UpdateContext, Vertex, Window,
	pipelines::SimplePipeline,
};


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

	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		if let Some(scene) = &mut self.scene {
			if let Some(pipeline) = self.pipeline.as_ref() {
				pipeline.apply(ctx.render_pass_mut());
				scene.draw(ctx);
			}
		}
	}

	fn update(&mut self, ctx: &mut UpdateContext) {
		let _dt = ctx.dt().as_secs_f32();
	}

	fn event(&mut self, _event: &Event) {}
}

fn main() {
	env_logger::init();
	Window::new().run(CubeApp::new());
}
