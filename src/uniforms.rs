use std::collections::HashMap;

use cgmath::{Matrix2, Matrix3, Matrix4, Point3, Vector3};

#[derive(Debug, Clone)]
pub enum UniformValue {
	Bool(bool),
	Int(i32),
	UnsignedInt(u32),
	Float(f32),
	Mat2([[f32; 2]; 2]),
	Mat3([[f32; 3]; 3]),
	Mat4([[f32; 4]; 4]),
	Vec2([f32; 2]),
	Vec3([f32; 3]),
	Vec4([f32; 4]),
	IntVec2([i32; 2]),
	IntVec3([i32; 3]),
	IntVec4([i32; 4]),
	UnsignedIntVec2([u32; 2]),
	UnsignedIntVec3([u32; 3]),
	UnsignedIntVec4([u32; 4]),
}

pub struct UniformMap(pub(crate) HashMap<String, UniformValue>);

impl UniformMap {
	pub fn new() -> Self {
		Self(HashMap::new())
	}

	pub fn insert(&mut self, name: &str, value: impl AsUniformValue) {
		self.0.insert(name.into(), value.as_uniform_value());
	}

	pub fn remove(&mut self, name: &str) -> Option<UniformValue> {
		self.0.remove(name)
	}
}

pub trait AsUniformValue {
	fn as_uniform_value(&self) -> UniformValue;
}

impl AsUniformValue for bool {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Bool(*self)
	}
}

impl AsUniformValue for f32 {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Float(*self)
	}
}

impl AsUniformValue for u32 {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::UnsignedInt(*self)
	}
}

impl AsUniformValue for i32 {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Int(*self)
	}
}

impl AsUniformValue for [f32; 2] {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Vec2(*self)
	}
}

impl AsUniformValue for [f32; 3] {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Vec3(*self)
	}
}

impl AsUniformValue for [f32; 4] {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Vec4(*self)
	}
}

impl AsUniformValue for Vector3<f32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Vec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Point3<f32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::Vec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Vector3<i32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::IntVec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Point3<i32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::IntVec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Vector3<u32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::UnsignedIntVec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Point3<u32> {
	fn as_uniform_value(&self) -> UniformValue {
		UniformValue::UnsignedIntVec3([self.x, self.y, self.z])
	}
}

impl AsUniformValue for Matrix2<f32> {
	fn as_uniform_value(&self) -> UniformValue {
		let mut mat = [[0.0; 2]; 2];
		mat[0] = [self.x.x, self.x.y];
		mat[1] = [self.y.x, self.y.y];
		UniformValue::Mat2(mat)
	}
}

impl AsUniformValue for Matrix3<f32> {
	fn as_uniform_value(&self) -> UniformValue {
		let mut mat = [[0.0; 3]; 3];
		mat[0] = [self.x.x, self.x.y, self.x.z];
		mat[1] = [self.y.x, self.y.y, self.y.z];
		mat[2] = [self.z.x, self.z.y, self.z.z];
		UniformValue::Mat3(mat)
	}
}

impl AsUniformValue for Matrix4<f32> {
	fn as_uniform_value(&self) -> UniformValue {
		let mut mat = [[0.0; 4]; 4];
		mat[0] = [self.x.x, self.x.y, self.x.z, self.x.w];
		mat[1] = [self.y.x, self.y.y, self.y.z, self.y.w];
		mat[2] = [self.z.x, self.z.y, self.z.z, self.z.w];
		mat[3] = [self.w.x, self.w.y, self.w.z, self.w.w];
		UniformValue::Mat4(mat)
	}
}

impl AsUniformValue for UniformValue {
	fn as_uniform_value(&self) -> UniformValue {
		self.clone()
	}
}
