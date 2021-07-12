use crate::{DrawContext, Drawable, MountContext, Vertex};
use cgmath::{Matrix4, Point3, SquareMatrix};
use std::error::Error;
use std::mem::size_of;
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	VertexFormat::{Float32x3, Float32x4},
};

#[repr(C)]
pub struct SimpleVertex {
	pub position: Point3<f32>,
	pub color: (f32, f32, f32, f32),
}

impl Vertex for SimpleVertex {
	fn buffer_layout<'a>() -> ::wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as _,
			step_mode: wgpu::InputStepMode::Vertex,
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

pub struct Mesh<V: Vertex + 'static> {
	vertices: Vec<V>,
	transform: Matrix4<f32>,
	buffer: Option<wgpu::Buffer>,
	uploaded: bool,
}

impl Mesh<SimpleVertex> {
	pub fn cube(size: f32) -> Self {
		let vertices: Vec<SimpleVertex> = CUBE_VERTICES
			.chunks(3)
			.map(|v| SimpleVertex {
				position: Point3::new(v[0] * size, v[1] * size, v[2] * size),
				color: (v[0] * 0.5 + 0.5, v[1] * 0.5 + 0.5, v[2] * 0.5 + 0.5, 0.7),
			})
			.collect();
		Self::new(vertices)
	}
}

impl<V: Vertex> Drawable for Mesh<V> {
	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>) {
		if !self.uploaded {
			self.upload(ctx.device()).unwrap();
		}

		let len = self.vertices.len() as u32;
		if let Some(buffer) = self.buffer.as_mut() {
			let render_pass = ctx.render_pass_mut();
			render_pass.set_vertex_buffer(0, buffer.slice(..));
			render_pass.draw(0..len, 0..1);
		}
	}

	fn mount(&mut self, ctx: &mut MountContext) {}

	fn unmount(&mut self, ctx: &mut MountContext) {
		self.free().unwrap();
	}
}

impl<V: Vertex> Mesh<V> {
	pub fn new(vertices: Vec<V>) -> Self {
		Self {
			uploaded: false,
			vertices,
			transform: Matrix4::identity(),
			buffer: None,
		}
	}

	pub fn upload(&mut self, device: &wgpu::Device) -> Result<(), Box<dyn Error>> {
		println!("Uploading mesh");
		self.uploaded = true;
		let contents = unsafe {
			let len = self.vertices.len() * size_of::<V>();
			let ptr = self.vertices.as_ptr() as *const u8;
			std::slice::from_raw_parts(ptr, len)
		};
		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Mesh Vertex Buffer"),
			contents,
			usage: wgpu::BufferUsage::VERTEX,
		});
		self.buffer = Some(vertex_buffer);
		Ok(())
	}

	pub fn free(&mut self) -> Result<(), Box<dyn Error>> {
		Ok(())
	}

	/// Get a reference to the mesh's vertices.
	pub fn vertices(&self) -> &[V] {
		self.vertices.as_slice()
	}

	/// Get a mutable reference to the mesh's vertices.
	pub fn vertices_mut(&mut self) -> &mut Vec<V> {
		&mut self.vertices
	}

	/// Get a reference to the mesh's transform.
	pub fn transform(&self) -> &Matrix4<f32> {
		&self.transform
	}

	/// Get a mutable reference to the mesh's transform.
	pub fn transform_mut(&mut self) -> &mut Matrix4<f32> {
		&mut self.transform
	}

	/// Set the mesh's transform.
	pub fn set_transform(&mut self, transform: Matrix4<f32>) {
		self.transform = transform;
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const CUBE_VERTICES: [f32; 108] = [
	// Far
	-1.0, 1.0, -1.0,
	1.0, 1.0, -1.0,
	-1.0, -1.0, -1.0,

	1.0, -1.0, -1.0,
	-1.0, -1.0, -1.0,
	1.0, 1.0, -1.0,

	// Near
	1.0, 1.0, 1.0,
	-1.0, 1.0, 1.0,
	1.0, -1.0, 1.0,

	-1.0, -1.0, 1.0,
	1.0, -1.0, 1.0,
	-1.0, 1.0, 1.0,

	// Left
	-1.0, 1.0, 1.0,
	-1.0, 1.0, -1.0,
	-1.0, -1.0, 1.0,

	-1.0, -1.0, -1.0,
	-1.0, -1.0, 1.0,
	-1.0, 1.0, -1.0,

	// Right
	1.0, 1.0, -1.0,
	1.0, 1.0, 1.0,
	1.0, -1.0, -1.0,

	1.0, -1.0, 1.0,
	1.0, -1.0, -1.0,
	1.0, 1.0, 1.0,

	// Top
	1.0, 1.0, -1.0,
	-1.0, 1.0, -1.0,
	1.0, 1.0, 1.0,

	-1.0, 1.0, 1.0,
	1.0, 1.0, 1.0,
	-1.0, 1.0, -1.0,

	// Bottom
	1.0, -1.0, -1.0,
	1.0, -1.0, 1.0,
	-1.0, -1.0, -1.0,

	-1.0, -1.0, 1.0,
	-1.0, -1.0, -1.0,
	1.0, -1.0, 1.0,
];
