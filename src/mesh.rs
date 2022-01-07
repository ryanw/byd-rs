use crate::Vertex;
use cgmath::{Matrix4, Point3, SquareMatrix};
use std::mem::size_of;
use wgpu::VertexFormat::{Float32x3, Float32x4};

pub type Color = u32;

pub trait Material {}

#[derive(Clone)]
pub struct Geometry {}

#[derive(Clone)]
pub struct BasicMaterial {
	color: Color,
}

#[derive(Clone)]
pub struct Mesh {
	geometry: Geometry,
	material: BasicMaterial,
	pub transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone)]
pub struct SimpleVertex {
	pub position: Point3<f32>,
	pub color: (f32, f32, f32, f32),
}

impl Geometry {
	pub fn new() -> Self {
		Self {}
	}

	pub fn cube() -> Self {
		Self {}
	}
}

impl BasicMaterial {
	pub fn new(color: Color) -> Self {
		Self { color }
	}
}

impl Material for BasicMaterial {}

impl Mesh {
	pub fn new(geometry: Geometry, material: BasicMaterial) -> Self {
		Self {
			geometry,
			material,
			transform: Matrix4::identity(),
		}
	}
}

impl Vertex for SimpleVertex {
	fn buffer_layout<'a>() -> ::wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as _,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: size_of::<Point3<f32>>() as _,
					shader_location: 1,
					format: Float32x4,
				},
			],
		}
	}
}
