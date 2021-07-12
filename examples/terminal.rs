use byd::{
	Actor, App, AttachContext, DrawContext, Event, Material, Mesh, RasterScene, Term, UpdateContext,
};

struct TermApp {
	scene: Option<RasterScene>,
}

impl TermApp {
	pub fn new() -> Self {
		Self { scene: None }
	}
}

impl App for TermApp {
	fn attach(&mut self, _ctx: &mut AttachContext) {
		let mut scene = RasterScene::new();
		let cube = Actor {
			geometry: Box::new(Mesh::cube(1.0)),
			material: Material::default(),
		};

		scene.add(cube);
		self.scene = Some(scene);
	}

	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		if let Some(scene) = &mut self.scene {
			scene.draw(ctx);
		}
	}

	fn update(&mut self, ctx: &mut UpdateContext) {
		let _dt = ctx.dt().as_secs_f32();
	}

	fn event(&mut self, _event: &Event) {}
}

fn main() {
	env_logger::init();
	Term::new().run(TermApp::new());
}
