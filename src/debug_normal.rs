use crate::{
	Color, LineMaterial, Material, MountContext, RenderContext, SceneObject, SimpleVertex,
};
use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3};
use std::mem::size_of_val;
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct DebugNormals {
	lines: Vec<[SimpleVertex; 2]>,
	vertex_buffer: Option<wgpu::Buffer>,
	dirty: bool,
	material: LineMaterial,
	pub transform: Matrix4<f32>,
}

impl DebugNormals {
	pub fn new() -> Self {
		Self {
			lines: vec![],
			vertex_buffer: None,
			dirty: true,
			transform: Matrix4::identity(),
			material: LineMaterial::new(),
		}
	}

	pub fn set_vertices(&mut self, vertices: &[SimpleVertex]) {
		self.lines = vertices
			.iter()
			.map(|v| {
				[
					SimpleVertex {
						position: v.position.clone(),
						..Default::default()
					},
					SimpleVertex {
						position: Point3::new(
							v.position.x + v.normal.x * 0.5,
							v.position.y + v.normal.y * 0.5,
							v.position.z + v.normal.z * 0.5,
						),
						..Default::default()
					},
				]
			})
			.collect();
		self.dirty = true;
	}

	fn allocate(&mut self, device: &wgpu::Device) {
		self.free();
		let contents = bytemuck::cast_slice(&self.lines);

		log::debug!(
			"Allocating DebugNormals vertex buffer ({} lines / {} bytes)",
			self.lines.len(),
			size_of_val(contents)
		);

		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("DebugNormals Vertex Buffer"),
			contents,
			usage: wgpu::BufferUsages::VERTEX,
		});

		self.vertex_buffer = Some(vertex_buffer);
	}

	fn free(&mut self) {
		if let Some(buffer) = self.vertex_buffer.take() {
			log::debug!("Freeing DebugNormals vertex buffer");
			buffer.destroy();
		}
	}
}

impl SceneObject for DebugNormals {
	fn render<'a>(&'a mut self, ctx: &mut RenderContext<'a>) {
		if self.dirty {
			self.allocate(ctx.device);
			self.dirty = false;
		}

		if let Some(buffer) = self.vertex_buffer.as_ref() {
			let render_pass = &mut ctx.render_pass;
			let len = self.lines.len() as u32 * 2;
			render_pass.set_vertex_buffer(0, buffer.slice(..));
			render_pass.draw(0..len, 0..1);
		}
	}

	fn mount(&mut self, ctx: &mut MountContext) {
		self.allocate(ctx.device);
		self.dirty = false;
	}

	fn unmount(&mut self, _ctx: &mut MountContext) {
		self.free();
	}

	fn transform(&self) -> Matrix4<f32> {
		self.transform.clone()
	}

	fn material(&self) -> Option<&dyn Material> {
		Some(&self.material)
	}
}
