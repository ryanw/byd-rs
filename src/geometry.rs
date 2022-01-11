use crate::Vertex;
use std::{error::Error, fmt, mem::size_of_val};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

#[derive(Debug)]
pub struct GeometryError(String);
impl Error for GeometryError {}

impl fmt::Display for GeometryError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

pub struct Geometry<V: Vertex> {
	vertices: Vec<V>,
	vertex_buffer: Option<wgpu::Buffer>,
}

impl<V: Vertex> Clone for Geometry<V> {
	fn clone(&self) -> Self {
		Self {
			vertices: self.vertices.clone(),
			vertex_buffer: None,
		}
	}
}

impl<V: Vertex> Geometry<V> {
	pub fn new(vertices: Vec<V>) -> Self {
		Self {
			vertices,
			vertex_buffer: None,
		}
	}

	pub fn allocate(&mut self, device: &wgpu::Device) -> Result<(), GeometryError> {
		self.free()?;

		let contents = bytemuck::cast_slice(&self.vertices);

		log::debug!(
			"Allocating geometry vertex buffer ({} vertices / {} bytes)",
			self.vertices.len(),
			size_of_val(contents)
		);

		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Geometry Vertex Buffer"),
			contents,
			usage: wgpu::BufferUsages::VERTEX,
		});

		self.vertex_buffer = Some(vertex_buffer);

		Ok(())
	}

	pub fn free(&mut self) -> Result<(), GeometryError> {
		if let Some(buffer) = self.vertex_buffer.take() {
			log::debug!("Freeing geometry vertex buffer");
			buffer.destroy();
		}

		Ok(())
	}

	pub fn vertex_count(&self) -> usize {
		self.vertices.len()
	}

	/// Get a reference to the geometry's vertex buffer.
	pub fn vertex_buffer(&self) -> Option<&wgpu::Buffer> {
		self.vertex_buffer.as_ref()
	}

	/// Get a reference to the geometry's vertices.
	pub fn vertices(&self) -> &[V] {
		self.vertices.as_ref()
	}

	/// Get a mutable reference to the geometry's vertices.
	pub fn vertices_mut(&mut self) -> &mut Vec<V> {
		&mut self.vertices
	}
}

impl<V: Vertex> Drop for Geometry<V> {
	fn drop(&mut self) {
		let _ = self.free();
	}
}

impl<V: Vertex + From<&'static [f32; 3]>> Geometry<V> {
	pub fn cube() -> Self {
		let vertices: Vec<V> = CUBE_VERTICES.iter().map(|vert| V::from(&vert)).collect();
		Self {
			vertices,
			vertex_buffer: None,
		}
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const CUBE_VERTICES: [[f32; 3]; 36] = [
	// Far
	[-1.0, 1.0, -1.0],
	[1.0, 1.0, -1.0],
	[-1.0, -1.0, -1.0],

	[1.0, -1.0, -1.0],
	[-1.0, -1.0, -1.0],
	[1.0, 1.0, -1.0],

	// Near
	[1.0, 1.0, 1.0],
	[-1.0, 1.0, 1.0],
	[1.0, -1.0, 1.0],

	[-1.0, -1.0, 1.0],
	[1.0, -1.0, 1.0],
	[-1.0, 1.0, 1.0],

	// Left
	[-1.0, 1.0, 1.0],
	[-1.0, 1.0, -1.0],
	[-1.0, -1.0, 1.0],

	[-1.0, -1.0, -1.0],
	[-1.0, -1.0, 1.0],
	[-1.0, 1.0, -1.0],

	// Right
	[1.0, 1.0, -1.0],
	[1.0, 1.0, 1.0],
	[1.0, -1.0, -1.0],

	[1.0, -1.0, 1.0],
	[1.0, -1.0, -1.0],
	[1.0, 1.0, 1.0],

	// Top
	[1.0, 1.0, -1.0],
	[-1.0, 1.0, -1.0],
	[1.0, 1.0, 1.0],

	[-1.0, 1.0, 1.0],
	[1.0, 1.0, 1.0],
	[-1.0, 1.0, -1.0],

	// Bottom
	[1.0, -1.0, -1.0],
	[1.0, -1.0, 1.0],
	[-1.0, -1.0, -1.0],

	[-1.0, -1.0, 1.0],
	[-1.0, -1.0, -1.0],
	[1.0, -1.0, 1.0],
];
