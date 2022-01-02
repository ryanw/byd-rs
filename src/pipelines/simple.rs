use crate::{SimpleVertex, Vertex};

pub struct SimplePipeline {
	render_pipeline: wgpu::RenderPipeline,
	camera_bind_group_layout: wgpu::BindGroupLayout,
}

impl SimplePipeline {
	pub fn new(device: &wgpu::Device) -> Self {
		// Uniforms
		let camera_bind_group_layout =
			device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					// Camera
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None,
						},
						count: None,
					},
				],
				label: Some("Camera Bind Group Layout"),
			});

		// Shader
		log::debug!("Creating Simple shader");
		let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some("Simple Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/simple.wgsl").into()),
		});

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&camera_bind_group_layout],
			push_constant_ranges: &[],
		});

		log::debug!("Creating pipeline");
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Cube Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader_module,
				entry_point: "vs_main",
				buffers: &[SimpleVertex::buffer_layout()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader_module,
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: wgpu::TextureFormat::Bgra8UnormSrgb, // FIXME ctx.swapchain_format(),
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				conservative: false,
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			multiview: None,
		});

		Self {
			render_pipeline: pipeline,
			camera_bind_group_layout,
		}
	}

	pub fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_pipeline(&self.render_pipeline);
	}

	/// Get a reference to the simple pipeline's bind group layout.
	pub fn camera_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.camera_bind_group_layout
	}
}
