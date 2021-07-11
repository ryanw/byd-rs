use crate::{SimpleVertex, Vertex};

pub struct SimplePipeline {
	render_pipeline: wgpu::RenderPipeline,
}

impl SimplePipeline {
	pub fn new(device: &wgpu::Device) -> Self {
		log::debug!("Creating vertex shader");
		let vs_module =
			device.create_shader_module(&wgpu::include_spirv!("../../shaders/simple.vert.spv"));
		log::debug!("Creating fragment shader");
		let fs_module =
			device.create_shader_module(&wgpu::include_spirv!("../../shaders/simple.frag.spv"));

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[],
			push_constant_ranges: &[],
		});

		log::debug!("Creating pipeline");
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Cube Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &vs_module,
				entry_point: "main",
				buffers: &[SimpleVertex::buffer_layout()],
			},
			fragment: Some(wgpu::FragmentState {
				module: &fs_module,
				entry_point: "main",
				targets: &[wgpu::ColorTargetState {
					format: wgpu::TextureFormat::Bgra8UnormSrgb, // FIXME ctx.swapchain_format(),
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrite::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				//cull_mode: Some(wgpu::Face::Back),
				cull_mode: None,
				clamp_depth: false,
				conservative: false,
				polygon_mode: wgpu::PolygonMode::Fill,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
		});

		Self {
			render_pipeline: pipeline,
		}
	}

	pub fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_pipeline(&self.render_pipeline);
	}
}

