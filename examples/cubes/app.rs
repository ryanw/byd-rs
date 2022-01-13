use byd::{
	BasicMaterial, Camera, Color, Event, FreeCamera, Geometry, Mesh, MouseButton, Renderer, Scene,
	SimpleVertex, Window,
};
use cgmath::{Euler, Matrix4, Rad, Vector3};

pub struct App {
	window: Option<Window>,
	scene: Scene,
	camera: FreeCamera,
	renderer: Renderer,

	cube: Mesh<SimpleVertex>,
	cube_ids: Vec<usize>,
}

impl App {
	pub async fn new(width: u32, height: u32) -> Self {
		let window = Window::new(width, height);
		let mut renderer = Renderer::new(width, height).await;
		renderer.attach(&window);
		let scene = Scene::new();
		let camera = FreeCamera::new();
		let mut cube: Mesh<SimpleVertex> = Mesh::new(
			Geometry::cube(),
			BasicMaterial::new(Color::new(1.0, 0.0, 1.0, 1.0)),
		);

		// Calculate normals
		// FIXME Geometry should do this
		for tri in cube.geometry_mut().vertices_mut().chunks_mut(3) {
			let u = tri[1].position - tri[0].position;
			let v = tri[2].position - tri[0].position;

			let normal = u.cross(v);
			tri[0].normal = normal;
			tri[1].normal = normal;
			tri[2].normal = normal;
		}

		Self {
			window: Some(window),
			scene,
			camera,
			renderer,

			cube,
			cube_ids: vec![],
		}
	}
}

impl App {
	pub fn add_cube(&mut self, x: f32, y: f32, z: f32) {
		let mut cube = self.cube.clone();
		cube.transform = Matrix4::from_translation(Vector3::new(x, y, z));
		cube.material.color = Color::new(
			rand::random::<f32>(),
			rand::random::<f32>(),
			rand::random::<f32>(),
			1.0,
		);
		self.cube_ids.push(self.scene.add(cube));
	}

	pub fn update(&mut self, dt: f32) {
		for id in &self.cube_ids {
			self.scene
				.with_object_mut(*id, |cube: &mut Mesh<SimpleVertex>| {
					cube.transform = cube.transform
						* Matrix4::from(Euler::new(Rad(0.0), Rad(1.0 * dt), Rad(0.623 * dt)));
				});
		}
	}

	pub fn render(&mut self, _dt: f32) {
		if let Err(error) = self.renderer.render(&mut self.scene, &self.camera) {
			log::error!("Error rendering scene: {:?}", error);
		}
	}

	pub fn run(mut self) {
		for _ in 0..7 {
			self.add_cube(
				(rand::random::<f32>() - 0.5) * 20.0,
				(rand::random::<f32>() - 0.5) * 20.0,
				rand::random::<f32>() * 20.0,
			);
		}

		let window = self.window.take().unwrap();
		window.run(move |event, _| match event {
			Event::MouseDown(MouseButton::Left, _x, _y) => {
				self.add_cube(
					(rand::random::<f32>() - 0.5) * 20.0,
					(rand::random::<f32>() - 0.5) * 20.0,
					rand::random::<f32>() * 20.0,
				);
			}
			Event::MouseDown(MouseButton::Right, _x, _y) => {
				if let Some(id) = self.cube_ids.pop() {
					self.scene.remove(id);
				}
			}
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
