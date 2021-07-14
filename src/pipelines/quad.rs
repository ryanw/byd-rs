use crate::Vertex;
use cgmath::Point2;
use std::mem::size_of;
use wgpu::{
	util::{BufferInitDescriptor, DeviceExt},
	VertexFormat::Float32x2,
};

#[repr(C)]
pub struct QuadVertex {
	pub position: Point2<f32>,
}

impl Vertex for QuadVertex {
	fn buffer_layout<'a>() -> ::wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: size_of::<Self>() as _,
			step_mode: wgpu::InputStepMode::Vertex,
			attributes: &[wgpu::VertexAttribute {
				offset: 0,
				shader_location: 0,
				format: Float32x2,
			}],
		}
	}
}

pub struct QuadPipeline {
	render_pipeline: wgpu::RenderPipeline,
	bind_group_layout: wgpu::BindGroupLayout,
	bind_group: wgpu::BindGroup,
	quad_buffer: wgpu::Buffer,
}

impl QuadPipeline {
	pub fn new(device: &wgpu::Device, texture_view: &wgpu::TextureView) -> Self {
		// Uniforms
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						multisampled: false,
						view_dimension: wgpu::TextureViewDimension::D2,
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::Sampler {
						comparison: false,
						filtering: true,
					},
					count: None,
				},
			],
			label: Some("Quad Bind Group Layout"),
		});

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
			],
			label: Some("Quad Bind Group"),
		});

		// Shader
		log::debug!("Creating vertex shader");
		let vs_module =
			device.create_shader_module(&wgpu::include_spirv!("../../shaders/quad.vert.spv"));
		log::debug!("Creating fragment shader");
		let fs_module =
			device.create_shader_module(&wgpu::include_spirv!("../../shaders/quad.frag.spv"));

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		log::debug!("Creating pipeline");
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Quad Render Pipeline"),
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &vs_module,
				entry_point: "main",
				buffers: &[QuadVertex::buffer_layout()],
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
				topology: wgpu::PrimitiveTopology::TriangleStrip,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
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

		// Quad mesh
		let vertices = [
			QuadVertex {
				position: Point2::new(-1.0, -1.0),
			},
			QuadVertex {
				position: Point2::new(1.0, -1.0),
			},
			QuadVertex {
				position: Point2::new(-1.0, 1.0),
			},
			QuadVertex {
				position: Point2::new(1.0, 1.0),
			},
		];
		let contents = unsafe {
			let len = vertices.len() * size_of::<QuadVertex>();
			let ptr = vertices.as_ptr() as *const u8;
			std::slice::from_raw_parts(ptr, len)
		};

		let quad_buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Quad Vertex Buffer"),
			contents,
			usage: wgpu::BufferUsage::VERTEX,
		});

		Self {
			render_pipeline: pipeline,
			bind_group,
			bind_group_layout,
			quad_buffer,
		}
	}

	pub fn set_texture_view(&mut self, device: &wgpu::Device, texture_view: &wgpu::TextureView) {
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &self.bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
			],
			label: Some("Quad Bind Group"),
		});
	}

	pub fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_pipeline(&self.render_pipeline);
	}

	/// Get a reference to the simple pipeline's bind group layout.
	pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn draw_quad<'a>(&'a mut self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_bind_group(0, &self.bind_group, &[]);
		self.apply(render_pass);
		render_pass.set_vertex_buffer(0, self.quad_buffer.slice(..));
		render_pass.draw(0..4, 0..1);
	}
}
