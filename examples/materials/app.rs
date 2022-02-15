use byd::{
	BasicMaterial, Camera, Color, CreateColor, CustomMaterial, Event, FreeCamera, Geometry, Mesh,
	Renderer, Scene, SimpleProgram, SimpleVertex, Vertex, Window,
};
use byd_derive::CastBytes;
use cgmath::{Euler, Matrix4, Point2, Point3, Rad, Vector3};
use std::mem;
use wgpu::VertexFormat::{Float32x2, Float32x3, Float32x4};

#[derive(Copy, Clone, Debug, CastBytes)]
#[repr(C)]
struct ColorVertex {
	position: Point3<f32>,
	color: Color,
}

#[derive(Copy, Clone, Debug, CastBytes)]
#[repr(C)]
struct TextureVertex {
	position: Point3<f32>,
	uv: Point2<f32>,
}

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
		let mut scene = Scene::new();
		let camera = FreeCamera::new();

		let color_pipeline: SimpleProgram<ColorVertex> =
			SimpleProgram::new().shader(include_str!("./shaders/color.wgsl"));
		let color_pipeline_id = scene.add_program(color_pipeline);
		let mut color_cube: Mesh<ColorVertex> =
			Mesh::new(Geometry::cube(), CustomMaterial::new(color_pipeline_id));
		color_cube.transform = Matrix4::from_translation(Vector3::new(-2.0, 0.0, 10.0))
			* Matrix4::from(Euler::new(Rad(0.0), Rad(1.0), Rad(0.623)));
		scene.add(color_cube);

		let texture_pipeline: SimpleProgram<TextureVertex> =
			SimpleProgram::new().shader(include_str!("./shaders/texture.wgsl"));
		let texture_pipeline_id = scene.add_program(texture_pipeline);
		let mut texture_cube: Mesh<TextureVertex> =
			Mesh::new(Geometry::cube(), CustomMaterial::new(texture_pipeline_id));
		texture_cube.transform = Matrix4::from_translation(Vector3::new(2.0, 0.0, 10.0))
			* Matrix4::from(Euler::new(Rad(0.0), Rad(-1.0), Rad(0.623)));
		scene.add(texture_cube);

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

impl Vertex for ColorVertex {
	fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as _,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<Point3<f32>>() as _,
					shader_location: 1,
					format: Float32x4,
				},
			],
		}
	}
}

impl Vertex for TextureVertex {
	fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as _,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<Point3<f32>>() as _,
					shader_location: 1,
					format: Float32x2,
				},
			],
		}
	}
}

impl From<&[f32; 3]> for ColorVertex {
	fn from(position: &[f32; 3]) -> Self {
		let position = Point3::new(position[0], position[1], position[2]);
		Self {
			position,
			color: Color::hsl(rand::random::<f32>(), 1.0, 0.5),
		}
	}
}

impl From<&[f32; 3]> for TextureVertex {
	fn from(position: &[f32; 3]) -> Self {
		let position = Point3::new(position[0], position[1], position[2]);
		Self {
			position,
			uv: Point2::new(position[0], position[1]),
		}
	}
}
