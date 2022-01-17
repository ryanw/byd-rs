use crate::{Geometry, Material, MountContext, RenderContext, SceneObject, Vertex};
use byd_derive::CastBytes;
use cgmath::{EuclideanSpace, Matrix4, Point2, Point3, SquareMatrix, Vector3};
use std::mem::size_of;
use wgpu::VertexFormat::{Float32x2, Float32x3};

pub struct Mesh<V: Vertex> {
	geometry: Geometry<V>,
	pub material: Box<dyn Material>,
	pub transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, CastBytes, Debug)]
pub struct SimpleVertex {
	pub position: Point3<f32>,
	pub normal: Vector3<f32>,
	pub uv: Point2<f32>,
}

impl Default for SimpleVertex {
	fn default() -> Self {
		Self {
			position: Point3::new(0.0, 0.0, 0.0),
			normal: Vector3::new(0.0, 0.0, 0.0),
			uv: Point2::new(0.0, 0.0),
		}
	}
}

impl From<[f32; 3]> for SimpleVertex {
	fn from(position: [f32; 3]) -> Self {
		Self::from(&position)
	}
}

impl From<&[f32; 3]> for SimpleVertex {
	fn from(position: &[f32; 3]) -> Self {
		let position = Point3::new(position[0], position[1], position[2]);
		Self {
			normal: position.to_vec(),
			position,
			uv: Point2::new(position[0], position[1]),
		}
	}
}

impl<V: Vertex> Mesh<V> {
	pub fn new(geometry: Geometry<V>, material: impl Material) -> Self {
		Self {
			geometry,
			material: Box::new(material),
			transform: Matrix4::identity(),
		}
	}

	pub fn transform_mut(&mut self) -> &mut Matrix4<f32> {
		&mut self.transform
	}

	/// Get a reference to the mesh's geometry.
	pub fn geometry(&self) -> &Geometry<V> {
		&self.geometry
	}

	/// Get a mutable reference to the mesh's geometry.
	pub fn geometry_mut(&mut self) -> &mut Geometry<V> {
		&mut self.geometry
	}

	/// Set the mesh's material.
	pub fn set_material(&mut self, material: impl Material) {
		self.material = Box::new(material);
	}
}

impl<V: Vertex> SceneObject for Mesh<V> {
	fn render<'a>(&'a mut self, ctx: &mut RenderContext<'a>) {
		if let Some(buffer) = self.geometry.vertex_buffer() {
			let render_pass = &mut ctx.render_pass;
			let len = self.geometry.vertex_count() as u32;
			render_pass.set_vertex_buffer(0, buffer.slice(..));
			render_pass.draw(0..len, 0..1);
		}
	}

	fn mount(&mut self, ctx: &mut MountContext) {
		log::debug!("Mesh mounted");
		self.geometry
			.allocate(ctx.device)
			.expect("Failed to allocate mesh geometry");
	}

	fn unmount(&mut self, _ctx: &mut MountContext) {
		log::debug!("Mesh unmounted");
		self.geometry.free().expect("Failed to free mesh geometry");
	}

	fn transform(&self) -> Matrix4<f32> {
		self.transform.clone()
	}

	fn material(&self) -> &dyn Material {
		&*self.material
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
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: (size_of::<Point3<f32>>() + size_of::<Vector3<f32>>()) as _,
					shader_location: 2,
					format: Float32x2,
				},
			],
		}
	}
}
