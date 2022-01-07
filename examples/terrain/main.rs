use byd::{BasicMaterial, Event, FreeCamera, Geometry, Mesh, Renderer, Scene, Window};
use cgmath::{Euler, Matrix4, Rad};
use futures::executor::block_on;
use std::cell::RefCell;
use std::rc::Rc;

const SIZE: (u32, u32) = (1280, 720);

async fn async_main() {
	let window = Window::new(SIZE.0, SIZE.1);
	let mut renderer = Renderer::new(SIZE.0, SIZE.1).await;
	renderer.attach(&window);
	let mut scene = Scene::new();
	let camera = FreeCamera::new();
	let cube = Mesh::new(Geometry::cube(), BasicMaterial::new(0xff00ff));
	let cube_ids = vec![
		scene.add(cube.clone()),
		scene.add(cube.clone()),
		scene.add(cube.clone()),
	];

	let scene = Rc::new(RefCell::new(scene));
	let update = {
		let scene = scene.clone();
		move |dt| {
			for id in &cube_ids {
				if let Some(cube) = scene.borrow_mut().get_mut(*id) {
					cube.transform = cube.transform
						* Matrix4::from(Euler::new(Rad(0.0), Rad(1.0 * dt), Rad(0.0)));
				}
			}
		}
	};

	let mut draw = {
		let scene = scene.clone();
		move |_| {
			renderer.render(&*scene.borrow(), &camera);
		}
	};

	window.run(move |event, _| match event {
		Event::MouseDown(button, x, y) => {
			log::debug!("MOUSE DOWN! {:?} - {}x{}", button, x, y);
		}
		Event::Draw(elapsed) => {
			let dt = elapsed.as_secs_f32();
			update(dt);
			draw(dt);
		}
		_ => {}
	});
}

fn main() {
	env_logger::init();
	block_on(async_main());
}
