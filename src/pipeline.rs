pub trait Pipeline {
	fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
	fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}
