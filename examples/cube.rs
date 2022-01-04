use byd::{
	Actor, App, AttachContext, DrawContext, Event, Material, Mesh, RasterScene, UpdateContext,
	Window,
};
use cgmath::{Euler, Matrix4, Rad, SquareMatrix, Vector3};

struct CubeApp {
	scene: RasterScene,
	rotation: Euler<Rad<f32>>,
}

impl CubeApp {
	pub fn new() -> Self {
		Self {
			scene: RasterScene::new(),
			rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
		}
	}
}

impl App for CubeApp {
	fn attach(&mut self, _ctx: &mut AttachContext) {
		let cube = Actor {
			geometry: Box::new(Mesh::cube(0.5)),
			material: Material::default(),
			transform: Matrix4::identity(),
		};
		self.scene.add(cube);

		let cube = Actor {
			geometry: Box::new(Mesh::cube(0.5)),
			material: Material::default(),
			transform: Matrix4::identity(),
		};
		self.scene.add(cube);

		let cube = Actor {
			geometry: Box::new(Mesh::cube(0.5)),
			material: Material::default(),
			transform: Matrix4::identity(),
		};
		self.scene.add(cube);
	}

	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		self.scene.draw(ctx);
	}

	fn update(&mut self, ctx: &mut UpdateContext) {
		let dt = ctx.dt().as_secs_f32();
		self.rotation.y += Rad(5.0) * dt;
		self.rotation.x += Rad(5.0) * dt;
		for (id, pod) in self.scene.actors_mut() {
			let y = -3.0 + *id as f32 * 2.0;
			pod.actor.transform = Matrix4::from_translation(Vector3::new(0.0, y, 0.5))
			* Matrix4::from(self.rotation);
		}
	}

	fn event(&mut self, _event: &Event) {}
}

fn main() {
	env_logger::init();
	Window::new().run(CubeApp::new());
}
