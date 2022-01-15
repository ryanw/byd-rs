use std::collections::HashSet;

use crate::Terrain;
use byd::{
	Camera, Event, FreeCamera, Key, MouseButton, Renderer, Scene, Texture, TextureMaterial, Window,
};
use cgmath::{Matrix4, Vector3};

pub struct App {
	window: Option<Window>,
	scene: Scene,
	camera: FreeCamera,
	camera_velocity: Vector3<f32>,
	camera_dampening: Vector3<f32>,
	renderer: Renderer,
	terrain: Terrain,
	terrain_id: usize,
	held_keys: HashSet<Key>,
}

impl App {
	pub async fn new(width: u32, height: u32) -> Self {
		let window = Window::new(width, height);
		let mut renderer = Renderer::new(width, height).await;
		renderer.attach(&window);
		let scene = Scene::new();
		let terrain = Terrain::new();

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
			terrain,
			terrain_id: 0,
			held_keys: HashSet::with_capacity(16),
		}
	}
}

impl App {
	pub fn update(&mut self, dt: f32) {
		self.update_camera(dt);
	}

	pub fn render(&mut self, _dt: f32) {
		if let Err(error) = self.renderer.render(&mut self.scene, &self.camera) {
			log::error!("Error rendering scene: {:?}", error);
		}
	}

	pub fn run(mut self) {
		let grass_texture_id = self.scene.add_texture(
			Texture::from_image_bytes(include_bytes!("./grass.png"))
				.expect("Failed to load grass texture"),
		);

		let mut terrain = self.terrain.generate_mesh(0, 0);
		terrain.transform = Matrix4::from_translation(Vector3::new(0.0, 0.0, 50.0));
		terrain
			.material
			.downcast_mut::<TextureMaterial>()
			.unwrap()
			.texture_id = grass_texture_id;

		self.terrain_id = self.scene.add(terrain);

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
