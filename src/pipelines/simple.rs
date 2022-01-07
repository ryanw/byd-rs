use std::mem::size_of;

use super::Uniform;
use crate::{DrawContext, SimpleVertex, Vertex};
use cgmath::Matrix4;

const CAMERA_BINDING: u32 = 0;
const ACTOR_BINDING: u32 = 1;
const MAX_ACTORS: u64 = 1024;

pub struct CameraUniform {
	pub view: Matrix4<f32>,
	pub projection: Matrix4<f32>,
}

impl Uniform for CameraUniform {}

pub struct ActorUniform {
	pub model: Matrix4<f32>,
}
impl Uniform for ActorUniform {}

pub struct SimplePipeline {
	render_pipeline: wgpu::RenderPipeline,
	bind_group_layout: wgpu::BindGroupLayout,
	bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	actor_buffer: wgpu::Buffer,
}

impl SimplePipeline {
	pub fn new(device: &wgpu::Device) -> Self {
		// Uniforms
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("SimplePipeline Bind Group Layout"),
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
		log::debug!("Creating Simple shader");
		let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
			label: Some("Simple Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/simple.wgsl").into()),
		});

		log::debug!("Creating pipeline layout");
		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&bind_group_layout],
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

		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Uniform Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: uniform_alignment,
			mapped_at_creation: false,
		});

		let actor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Actor Uniform Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: MAX_ACTORS * uniform_alignment,
			mapped_at_creation: false,
		});

		let camera_uniform_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_uniform_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("SimplePipeline Bind Group"),
			layout: &bind_group_layout,
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						offset: 0,
						size: wgpu::BufferSize::new(camera_uniform_size),
					}),
				},
				// Actor
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &actor_buffer,
						offset: 0,
						size: wgpu::BufferSize::new(actor_uniform_size),
					}),
				},
			],
		});

		Self {
			render_pipeline: pipeline,
			bind_group_layout,
			bind_group,
			camera_buffer,
			actor_buffer,
		}
	}

	pub fn apply<'a>(&'a self, ctx: &mut DrawContext<'a>) {
		ctx.render_pass_mut().set_pipeline(&self.render_pipeline);
	}

	/// Get a reference to the simple pipeline's bind group layout.
	pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.bind_group_layout
	}

	pub fn set_camera(&self, ctx: &mut DrawContext, camera: &CameraUniform) {
		ctx.queue()
			.write_buffer(&self.camera_buffer, 0, camera.as_bytes());
	}

	pub fn set_actor(&self, ctx: &mut DrawContext, index: usize, actor: &ActorUniform) {
		let offset = ctx.uniform_alignment(index);

		ctx.queue()
			.write_buffer(&self.actor_buffer, offset as _, actor.as_bytes());
	}

	pub fn bind_actor<'a>(&'a self, ctx: &mut DrawContext<'a>, index: usize) {
		let offset = ctx.uniform_alignment(index);
		ctx.render_pass_mut()
			.set_bind_group(0, &self.bind_group, &[offset]);
	}
}
