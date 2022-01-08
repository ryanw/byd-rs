pub trait Vertex: bytemuck::Pod {
	fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}
