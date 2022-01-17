use byd::{
	Camera, Event, FreeCamera, Geometry, Gltf, Key, Mesh, MouseButton, PrimitiveVertex, Renderer,
	Scene, SimpleVertex, Texture, TextureMaterial, Window,
};
use cgmath::{Euler, Matrix4, Point2, Point3, Rad, Vector3};
use std::{collections::HashSet, error::Error};

pub struct App {
	window: Option<Window>,
	scene: Scene,
	camera: FreeCamera,
	camera_velocity: Vector3<f32>,
	camera_dampening: Vector3<f32>,
	renderer: Renderer,
	held_keys: HashSet<Key>,
	objects: Vec<usize>,
	textures: Vec<usize>,
}

impl App {
	pub async fn new(width: u32, height: u32) -> Self {
		let window = Window::new(width, height);
		let mut renderer = Renderer::new(width, height).await;
		renderer.attach(&window);
		let scene = Scene::new();

		let mut camera = FreeCamera::new();
		camera.translate(0.0, 10.0, -10.0);
		camera.rotate(0.3, 0.0, 0.0);

		Self {
			window: Some(window),
			scene,
			camera,
			camera_velocity: Vector3::new(0.0, 0.0, 0.0),
			camera_dampening: Vector3::new(5.0, 5.0, 5.0),
			renderer,
			held_keys: HashSet::with_capacity(16),
			objects: vec![],
			textures: vec![],
		}
	}
}

impl App {
	fn update(&mut self, dt: f32) {
		self.update_camera(dt);

		for id in &self.objects {
			self.scene
				.with_object_mut(*id, |cube: &mut Mesh<PrimitiveVertex>| {
					cube.transform =
						Matrix4::from(Euler::new(Rad(0.0), Rad(1.0 * dt), Rad(0.0 * dt)))
							* cube.transform;
				});
		}
	}

	fn render(&mut self, _dt: f32) {
		if let Err(error) = self.renderer.render(&mut self.scene, &self.camera) {
			log::error!("Error rendering scene: {:?}", error);
		}
	}

	fn build_scene(&mut self) {
		self.build_floor();
		self.load_thingy().expect("Failed to load thingy mesh");
	}

	fn load_thingy(&mut self) -> Result<(), Box<dyn Error>> {
		let mut thingy = Gltf::load("./examples/loading/assets/duck/Duck.gltf")?;

		for texture in thingy.textures.drain(..) {
			self.textures.push(self.scene.add_texture(texture));
		}

		for (i, mut mesh) in thingy.meshes.drain(..).enumerate() {
			mesh.transform = Matrix4::from_translation(Vector3::new(0.0, 3.0, 0.0))
				* Matrix4::from_scale(3.0)
				* mesh.transform;
			let texture_id = self.textures[thingy.mesh_textures.remove(&i).unwrap()];
			mesh.set_material(TextureMaterial::new(texture_id));
			self.objects.push(self.scene.add(mesh));
		}

		Ok(())
	}

	fn build_floor(&mut self) {
		let texture_id = self.scene.add_texture(
			Texture::from_image_bytes(include_bytes!("../../assets/checker.tif"))
				.expect("Failed to load texture"),
		);
		let mut floor = Mesh::new(
			Geometry::new(vec![
				SimpleVertex {
					position: Point3::new(1.0, 0.0, -1.0),
					uv: Point2::new(1.0, 1.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
				SimpleVertex {
					position: Point3::new(-1.0, 0.0, -1.0),
					uv: Point2::new(0.0, 1.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
				SimpleVertex {
					position: Point3::new(1.0, 0.0, 1.0),
					uv: Point2::new(1.0, 0.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
				SimpleVertex {
					position: Point3::new(-1.0, 0.0, 1.0),
					uv: Point2::new(0.0, 0.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
				SimpleVertex {
					position: Point3::new(1.0, 0.0, 1.0),
					uv: Point2::new(1.0, 0.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
				SimpleVertex {
					position: Point3::new(-1.0, 0.0, -1.0),
					uv: Point2::new(0.0, 1.0),
					normal: Vector3::new(0.0, 1.0, 0.0),
				},
			]),
			TextureMaterial::new(texture_id),
		);
		floor.transform =
			Matrix4::from_translation(Vector3::new(0.0, 3.0, 0.0)) * Matrix4::from_scale(50.0);

		self.objects = vec![self.scene.add(floor)];
	}

	pub fn run(mut self) {
		self.build_scene();
		//std::process::exit(0);

		let window = self.window.take().unwrap();
		let mut grabbed = false;

		window.run(move |event, ctx| match event {
			Event::KeyDown(Key::Space) => {
				self.held_keys = ctx.held_keys().clone();
				grabbed = !grabbed;
				if grabbed {
					ctx.grab_mouse();
				} else {
					ctx.release_mouse();
				}
			}
			Event::KeyDown(_) => self.held_keys = ctx.held_keys().clone(),
			Event::KeyUp(_) => self.held_keys = ctx.held_keys().clone(),
			Event::MouseMotion(x, y) => {
				self.camera.rotate(y / 500.0, x / 500.0, 0.0);
			}
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

	fn update_camera(&mut self, dt: f32) {
		self.dampen_camera(dt);
		let camera_speed = 50.0;

		if self.held_keys.contains(&Key::W) {
			self.camera_velocity.z = camera_speed;
		}
		if self.held_keys.contains(&Key::A) {
			self.camera_velocity.x = -camera_speed;
		}
		if self.held_keys.contains(&Key::S) {
			self.camera_velocity.z = -camera_speed;
		}
		if self.held_keys.contains(&Key::D) {
			self.camera_velocity.x = camera_speed;
		}
		if self.held_keys.contains(&Key::Q) {
			self.camera_velocity.y = -camera_speed;
		}
		if self.held_keys.contains(&Key::E) {
			self.camera_velocity.y = camera_speed;
		}
		self.camera.translate(
			self.camera_velocity.x * dt,
			self.camera_velocity.y * dt,
			self.camera_velocity.z * dt,
		);
	}

	fn dampen_camera(&mut self, dt: f32) {
		self.camera_velocity.x *= 1.0 - self.camera_dampening.x * dt;
		self.camera_velocity.y *= 1.0 - self.camera_dampening.y * dt;
		self.camera_velocity.z *= 1.0 - self.camera_dampening.z * dt;
		let min = 1.0;
		if self.camera_velocity.x.abs() < min {
			self.camera_velocity.x = 0.0;
		}
		if self.camera_velocity.y.abs() < min {
			self.camera_velocity.y = 0.0;
		}
		if self.camera_velocity.z.abs() < min {
			self.camera_velocity.z = 0.0;
		}
	}
}
