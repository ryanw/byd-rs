use byd::{Camera, Event, FreeCamera, MouseButton, Renderer, Scene, Window};

pub struct App {
	window: Option<Window>,
	scene: Scene,
	camera: FreeCamera,
	renderer: Renderer,
}

impl App {
	pub async fn new(width: u32, height: u32) -> Self {
		let window = Window::new(width, height);
		let mut renderer = Renderer::new(width, height).await;
		renderer.attach(&window);
		let scene = Scene::new();
		let camera = FreeCamera::new();

		Self {
			window: Some(window),
			scene,
			camera,
			renderer,
		}
	}
}

impl App {
	pub fn update(&mut self, _dt: f32) {}

	pub fn render(&mut self, _dt: f32) {
		if let Err(error) = self.renderer.render(&mut self.scene, &self.camera) {
			log::error!("Error rendering scene: {:?}", error);
		}
	}

	pub fn run(mut self) {
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
