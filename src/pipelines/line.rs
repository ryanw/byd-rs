use super::Uniform;
use crate::{Pipeline, SimpleVertex, TextureBuffer, Vertex};
use byd_derive::CastBytes;
use cgmath::{Matrix4, Vector4};

pub const CAMERA_BINDING: u32 = 0;
pub const ACTOR_BINDING: u32 = 1;

#[derive(Copy, Clone, CastBytes)]
pub struct CameraUniform {
	pub view: Matrix4<f32>,
	pub projection: Matrix4<f32>,
}

impl Uniform for CameraUniform {}

#[derive(Copy, Clone, CastBytes)]
pub struct ActorUniform {
	pub color: Vector4<f32>,
	pub model: Matrix4<f32>,
}
impl Uniform for ActorUniform {}

pub struct LinePipeline {
	render_pipeline: wgpu::RenderPipeline,
	bind_group_layout: wgpu::BindGroupLayout,
}

impl LinePipeline {
	pub fn new(device: &wgpu::Device) -> Self {
		// Uniforms
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("LinePipeline Bind Group Layout"),
			entries: &[
				// Camera
				wgpu::BindGroupLayoutEntry {
					binding: CAMERA_BINDING,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
				// Actor
				wgpu::BindGroupLayoutEntry {
					binding: ACTOR_BINDING,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: true,
						min_binding_size: None,
					},
					count: None,
				},
			],
		});

		// Shader
		log::debug!("Creating Line shader");
		let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some("Line Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/line.wgsl").into()),
		});

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		log::debug!("Creating pipeline");
		let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Line Render Pipeline"),
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
					format: wgpu::TextureFormat::Rgba8UnormSrgb, // FIXME ctx.swapchain_format(),
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL,
				}],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::LineList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Cw,
				cull_mode: None,
				conservative: false,
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
			},
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: !0,
				alpha_to_coverage_enabled: false,
			},
			depth_stencil: Some(wgpu::DepthStencilState {
				format: TextureBuffer::DEPTH_FORMAT,
				depth_write_enabled: true,
				depth_compare: wgpu::CompareFunction::Less,
				stencil: wgpu::StencilState::default(),
				bias: wgpu::DepthBiasState::default(),
			}),
			multiview: None,
		});

		Self {
			render_pipeline: pipeline,
			bind_group_layout,
		}
	}
}

impl Pipeline for LinePipeline {
	fn apply<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		render_pass.set_pipeline(&self.render_pipeline);
	}

	fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.bind_group_layout
	}
}
