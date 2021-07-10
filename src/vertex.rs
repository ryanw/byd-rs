pub trait Vertex {
	fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a>;
}
