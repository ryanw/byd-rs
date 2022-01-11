use crate::Terrain;
use byd::{
	Camera, DebugNormals, Event, FreeCamera, Mesh, MouseButton, Renderer, Scene, SceneObject,
	SimpleVertex, Window,
};
use cgmath::{Euler, Matrix4, Rad, Vector3};

pub struct App {
	window: Option<Window>,
	scene: Scene,
	camera: FreeCamera,
	renderer: Renderer,
	terrain: Terrain,
	terrain_id: usize,

	debug_normals_id: usize,
}

impl App {
	pub async fn new(width: u32, height: u32) -> Self {
		let window = Window::new(width, height);
		let mut renderer = Renderer::new(width, height).await;
		renderer.attach(&window);
		let scene = Scene::new();
		let terrain = Terrain::new();

		let mut camera = FreeCamera::new();
		camera.rotate(-0.3, 0.0, 0.0);

		Self {
			window: Some(window),
			scene,
			camera,
			renderer,
			terrain,
			terrain_id: 0,
			debug_normals_id: 0,
		}
	}
}

impl App {
	pub fn update(&mut self, dt: f32) {
		self.scene
			.with_object_mut(self.terrain_id, |obj: &mut Mesh<SimpleVertex>| {
				obj.transform = obj.transform
					* Matrix4::from(Euler::new(Rad(0.0), Rad(1.0 * dt), Rad(0.0 * dt)));
			});
		self.scene
			.with_object_mut(self.debug_normals_id, |obj: &mut DebugNormals| {
				obj.transform = obj.transform
					* Matrix4::from(Euler::new(Rad(0.0), Rad(1.0 * dt), Rad(0.0 * dt)));
			});
	}

	pub fn render(&mut self, _dt: f32) {
		if let Err(error) = self.renderer.render(&mut self.scene, &self.camera) {
			log::error!("Error rendering scene: {:?}", error);
		}
	}

	pub fn run(mut self) {
		let mut terrain = self.terrain.generate_mesh(0, 0);
		terrain.transform = Matrix4::from_translation(Vector3::new(0.0, -4.0, -20.0));

		let mut debug_normals = DebugNormals::new();
		debug_normals.transform = terrain.transform();
		debug_normals.set_vertices(terrain.geometry().vertices());

		self.terrain_id = self.scene.add(terrain);
		self.debug_normals_id = self.scene.add(debug_normals);

		let window = self.window.take().unwrap();
		window.run(move |event, _| match event {
			Event::MouseDown(MouseButton::Left, _x, _y) => {}
			Event::MouseDown(MouseButton::Right, _x, _y) => {}
			Event::Draw(elapsed) => {
				let dt = elapsed.as_secs_f32();
				self.update(dt);
				self.render(dt);
			}
			Event::WindowResize(width, height) => {
				log::debug!("Window resized {}x{}", width, height);
				self.renderer.resize(width, height);
				self.camera.resize(width as _, height as _);
			}
			_ => {}
		});
	}
}
