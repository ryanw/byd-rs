use crate::{BasicMaterial, Geometry, MountContext, RenderContext, SceneObject, Vertex};
use byd_derive::CastBytes;
use cgmath::{Matrix4, Point3, SquareMatrix};
use std::mem::size_of;
use wgpu::VertexFormat::{Float32x3, Float32x4};

#[derive(Clone)]
pub struct Mesh<V: Vertex> {
	geometry: Geometry<V>,
	material: BasicMaterial,
	pub transform: Matrix4<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, CastBytes)]
pub struct SimpleVertex {
	pub position: Point3<f32>,
	pub color: (f32, f32, f32, f32),
}

impl From<[f32; 3]> for SimpleVertex {
	fn from(position: [f32; 3]) -> Self {
		Self::from(&position)
	}
}

impl From<&[f32; 3]> for SimpleVertex {
	fn from(position: &[f32; 3]) -> Self {
		Self {
			position: Point3::new(position[0], position[1], position[2]),
			color: (1.0, 0.0, 0.0, 1.0),
		}
	}
}

impl<V: Vertex> Mesh<V> {
	pub fn new(geometry: Geometry<V>, material: BasicMaterial) -> Self {
		Self {
			geometry,
			material,
			transform: Matrix4::identity(),
		}
	}

	pub fn transform_mut(&mut self) -> &mut Matrix4<f32> {
		&mut self.transform
	}
}

impl<V: Vertex> SceneObject for Mesh<V> {
	fn render<'a>(&'a self, ctx: &mut RenderContext<'a>) {
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
