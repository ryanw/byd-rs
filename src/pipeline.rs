pub trait Pipeline {
	fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
	fn texture_bind_group_layout(&self) -> Option<&wgpu::BindGroupLayout> {
		None
	}
	fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
}
