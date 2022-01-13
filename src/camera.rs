use std::f32::consts::PI;

use cgmath::{Deg, EuclideanSpace, Euler, Matrix4, Point3, Rad, SquareMatrix, Transform, Vector3};

pub trait Camera {
	fn resize(&mut self, _width: f32, _height: f32) {}
	fn view(&self) -> Matrix4<f32>;
	fn projection(&self) -> Matrix4<f32> {
		Matrix4::identity()
	}
}

#[derive(Debug, Clone)]
pub struct FreeCamera {
	width: f32,
	height: f32,
	position: Point3<f32>,
	rotation: Euler<Rad<f32>>,
	projection: Matrix4<f32>,
}

impl FreeCamera {
	pub fn new() -> Self {
		let mut camera = Self {
			width: 1.0,
			height: 1.0,
			position: Point3::new(0.0, 0.0, 0.0),
			rotation: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
			projection: Matrix4::identity(),
		};

		camera.resize(1280.0, 720.0);

		camera
	}

	pub fn translate(&mut self, x: f32, y: f32, z: f32) {
		let trans = Matrix4::from_translation(Vector3::new(x, y, z));
		let rotate = self.rotation_matrix();
		let rotate_inv = rotate.inverse_transform().unwrap();

		let new_pos = (trans * rotate_inv).transform_point(self.position);
		self.position = rotate.transform_point(new_pos);
	}

	pub fn rotation_matrix(&self) -> Matrix4<f32> {
		let x: Matrix4<f32> = Euler::new(self.rotation.x, Rad(0.0), Rad(0.0)).into();
		let y: Matrix4<f32> = Euler::new(Rad(0.0), self.rotation.y, Rad(0.0)).into();
		let z: Matrix4<f32> = Euler::new(Rad(0.0), Rad(0.0), self.rotation.z).into();

		z * y * x
	}

	pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
		self.rotation = Euler::new(
			Rad(x) + self.rotation.x,
			Rad(y) + self.rotation.y,
			Rad(z) + self.rotation.z,
		);

		let min_tilt = Rad(-PI / 2.0 + 0.01);
		let max_tilt = Rad(PI / 2.0 - 0.01);
		if self.rotation.x < min_tilt {
			self.rotation.x = min_tilt;
		} else if self.rotation.x > max_tilt {
			self.rotation.x = max_tilt
		}
	}

	pub fn resolution(&self) -> (f32, f32) {
		(self.width, self.height)
	}

	/// Get a reference to the free camera's position.
	pub fn position(&self) -> &Point3<f32> {
		&self.position
	}

	/// Get a reference to the free camera's rotation.
	pub fn rotation(&self) -> &Euler<Rad<f32>> {
		&self.rotation
	}

	/// Get a mutable reference to the free camera's position.
	pub fn position_mut(&mut self) -> &mut Point3<f32> {
		&mut self.position
	}

	/// Get a mutable reference to the free camera's rotation.
	pub fn rotation_mut(&mut self) -> &mut Euler<Rad<f32>> {
		&mut self.rotation
	}

	/// Get a reference to the free camera's width.
	pub fn width(&self) -> f32 {
		self.width
	}

	/// Get a reference to the free camera's height.
	pub fn height(&self) -> f32 {
		self.height
	}
}

impl Camera for FreeCamera {
	fn view(&self) -> Matrix4<f32> {
		let translate: Matrix4<f32> = Matrix4::from_translation(self.position.to_vec());
		let rotate = self.rotation_matrix();

		(translate * rotate).inverse_transform().unwrap()
	}

	fn projection(&self) -> Matrix4<f32> {
		self.projection.clone()
	}

	fn resize(&mut self, width: f32, height: f32) {
		self.width = width;
		self.height = height;
		let aspect = width / height;
		let fov = 45.0;
		let near = 0.1;
		let far = 1000.0;
		// cgmath returns RH matrix, but we want LH, so we invert Z to flip it
		self.projection = cgmath::perspective(Deg(fov), aspect, near, far)
			* Matrix4::from_nonuniform_scale(1.0, 1.0, -1.0);
	}
}
