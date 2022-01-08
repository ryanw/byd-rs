use byd::{
	BasicMaterial, Event, FreeCamera, Geometry, Mesh, MouseButton, Renderer, Scene, SimpleVertex,
	Window,
};
use cgmath::{Euler, Matrix4, Rad, Vector3};
use futures::executor::block_on;
use rand::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

const SIZE: (u32, u32) = (1280, 720);

async fn async_main() {
	let window = Window::new(SIZE.0, SIZE.1);
	let mut renderer = Renderer::new(SIZE.0, SIZE.1).await;
	renderer.attach(&window);
	let scene = Scene::new();
	let camera = FreeCamera::new();
	let cube: Mesh<SimpleVertex> = Mesh::new(Geometry::cube(), BasicMaterial::new(0xff00ff));
	let cube_ids: Rc<RefCell<Vec<usize>>> = Rc::new(RefCell::new(vec![]));

	let scene = Rc::new(RefCell::new(scene));
	let renderer = Rc::new(RefCell::new(renderer));

	let update = {
		let scene = scene.clone();
		let cube_ids = cube_ids.clone();
		move |dt| {
			let mut scene = scene.borrow_mut();
			for id in cube_ids.borrow().iter() {
				scene.with_object_mut(*id, |cube: &mut Mesh<SimpleVertex>| {
					cube.transform = cube.transform
						* Matrix4::from(Euler::new(Rad(0.6 * dt), Rad(1.0 * dt), Rad(0.0)));
				});
			}
		}
	};

	let draw = {
		let scene = scene.clone();
		let renderer = renderer.clone();
		move |_| {
			if let Err(error) = renderer.borrow_mut().render(scene.borrow_mut(), &camera) {
				log::error!("Error rendering scene: {:?}", error);
			}
		}
	};

	window.run(move |event, _| match event {
		Event::MouseDown(MouseButton::Left, _x, _y) => {
			let mut cube = cube.clone();
			cube.transform = Matrix4::from_translation(Vector3::new(
				(rand::random::<f32>() - 0.5) * 20.0,
				(rand::random::<f32>() - 0.5) * 20.0,
				rand::random::<f32>() * -20.0,
			));
			cube_ids.borrow_mut().push(scene.borrow_mut().add(cube));
		}
		Event::MouseDown(MouseButton::Right, _x, _y) => {
			if let Some(id) = cube_ids.borrow_mut().pop() {
				scene.borrow_mut().remove(id);
			}
		}
		Event::Draw(elapsed) => {
			let dt = elapsed.as_secs_f32();
			update(dt);
			draw(dt);
		}
		Event::WindowResize(width, height) => {
			log::debug!("Window resized {}x{}", width, height);
			renderer.borrow_mut().resize(width, height);
		}
		_ => {}
	});
}

fn main() {
	env_logger::init();
	block_on(async_main());
}
