use std::{collections::HashMap, mem::size_of};

use crate::{
	pipelines::{
		ActorUniform, CameraUniform, CustomPipeline, ACTOR_BINDING, CAMERA_BINDING,
		SAMPLER_BINDING, TEXTURE_BINDING, TEXTURE_ENABLED_BINDING,
	},
	Camera, Pipeline, RenderContext, TextureBuffer, TextureID, Vertex,
};

const MAX_OBJECTS: u64 = 2048;

pub trait Program {
	fn compile(&mut self, ctx: &mut RenderContext);
	fn set_camera(&self, ctx: &mut RenderContext, camera: &dyn Camera);
	fn set_actor(&self, ctx: &mut RenderContext, index: u64, contents: ActorUniform);
	fn bind_actor<'a>(&'a self, ctx: &mut RenderContext<'a>, index: u64);
	fn bind_texture<'a>(&'a self, ctx: &mut RenderContext<'a>, id: TextureID);
	fn add_texture(&mut self, id: TextureID, device: &wgpu::Device, texture: &TextureBuffer);
}

struct ProgramState<P: Pipeline> {
	pipeline: P,
	bind_group: wgpu::BindGroup,
	texture_bind_groups: HashMap<TextureID, wgpu::BindGroup>,
	camera_buffer: wgpu::Buffer,
	actor_buffer: wgpu::Buffer,
	enabled_buffer: wgpu::Buffer,
}

impl<P: Pipeline> ProgramState<P> {
	pub fn set_camera(&self, ctx: &mut RenderContext, camera: &dyn Camera) {
		let contents = CameraUniform {
			view: camera.view(),
			projection: camera.projection(),
		};
		ctx.queue
			.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[contents]));
	}

	pub fn set_actor(&self, ctx: &mut RenderContext, index: u64, contents: ActorUniform) {
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		ctx.queue.write_buffer(
			&self.actor_buffer,
			offset as _,
			bytemuck::cast_slice(&[contents]),
		);
	}

	pub fn bind_actor<'a>(&'a self, ctx: &mut RenderContext<'a>, index: u64) {
		let render_pass = &mut ctx.render_pass;
		let uniform_alignment =
			ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
		let offset = (index * uniform_alignment) as wgpu::DynamicOffset;
		self.pipeline.apply(render_pass);
		render_pass.set_bind_group(0, &self.bind_group, &[offset]);
	}

	fn bind_texture<'a>(&'a self, ctx: &mut RenderContext<'a>, id: TextureID) {
		if let Some(texture) = self.texture_bind_groups.get(&id) {
			let uniform_alignment =
				ctx.device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
			let is_enabled_offset =
				(if id == 0 { 0 } else { uniform_alignment }) as wgpu::DynamicOffset;

			ctx.render_pass
				.set_bind_group(1, texture, &[is_enabled_offset]);
		} else {
			panic!("missing texture: {}", id);
		}
	}

	fn add_texture(&mut self, id: TextureID, device: &wgpu::Device, texture: &TextureBuffer) {
		log::debug!("Creating BindGroup for texture {}", id);
		self.texture_bind_groups.insert(
			id,
			device.create_bind_group(&wgpu::BindGroupDescriptor {
				label: Some("SimplePipeline Texture Bind Group"),
				layout: self
					.pipeline
					.texture_bind_group_layout()
					.expect("Missing texture bind group layout"),
				entries: &[
					wgpu::BindGroupEntry {
						binding: TEXTURE_ENABLED_BINDING,
						resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
							buffer: &self.enabled_buffer,
							size: wgpu::BufferSize::new(size_of::<u32>() as _),
							offset: 0,
						}),
					},
					wgpu::BindGroupEntry {
						binding: TEXTURE_BINDING,
						resource: wgpu::BindingResource::TextureView(&texture.view),
					},
					wgpu::BindGroupEntry {
						binding: SAMPLER_BINDING,
						resource: wgpu::BindingResource::Sampler(&texture.sampler),
					},
				],
			}),
		);
	}
}

pub struct SimpleProgram<V: Vertex> {
	state: Option<ProgramState<CustomPipeline<V>>>,
	source: String,
}

impl<V: Vertex> SimpleProgram<V> {
	pub fn new() -> Self {
		Self {
			state: None,
			source: "".into(),
		}
	}
	pub fn shader(mut self, source: &str) -> Self {
		self.source = source.into();
		self
	}
}

impl<V: Vertex> Program for SimpleProgram<V> {
	fn compile(&mut self, ctx: &mut RenderContext) {
		let device = ctx.device;
		let queue = &mut ctx.queue;
		let pipeline = CustomPipeline::new(device, &self.source);
		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: uniform_alignment,
			mapped_at_creation: false,
		});

		let actor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Actor Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: MAX_OBJECTS * uniform_alignment,
			mapped_at_creation: false,
		});

		let enabled_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Texture Enabled Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: 2 * uniform_alignment,
			mapped_at_creation: false,
		});

		queue.write_buffer(
			&enabled_buffer,
			0 * uniform_alignment,
			bytemuck::cast_slice(&[0]),
		);
		queue.write_buffer(
			&enabled_buffer,
			1 * uniform_alignment,
			bytemuck::cast_slice(&[0]),
		);

		let camera_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Program Bind Group"),
			layout: pipeline.bind_group_layout(),
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: CAMERA_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						size: wgpu::BufferSize::new(camera_size),
						offset: 0,
					}),
				},
				// Actors
				wgpu::BindGroupEntry {
					binding: ACTOR_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &actor_buffer,
						size: wgpu::BufferSize::new(actor_size),
						offset: 0,
					}),
				},
			],
		});

		self.state = Some(ProgramState {
			pipeline,
			bind_group,
			texture_bind_groups: HashMap::new(),
			camera_buffer,
			actor_buffer,
			enabled_buffer,
		});
	}

	fn set_camera(&self, ctx: &mut RenderContext, camera: &dyn Camera) {
		self.state.as_ref().map(|s| s.set_camera(ctx, camera));
	}

	fn set_actor(&self, ctx: &mut RenderContext, index: u64, contents: ActorUniform) {
		self.state
			.as_ref()
			.map(|s| s.set_actor(ctx, index, contents));
	}

	fn bind_actor<'a>(&'a self, ctx: &mut RenderContext<'a>, index: u64) {
		self.state.as_ref().map(|s| s.bind_actor(ctx, index));
	}

	fn bind_texture<'a>(&'a self, ctx: &mut RenderContext<'a>, id: TextureID) {
		self.state.as_ref().map(|s| s.bind_texture(ctx, id));
	}

	fn add_texture(&mut self, id: TextureID, device: &wgpu::Device, texture: &TextureBuffer) {
		self.state
			.as_mut()
			.map(|s| s.add_texture(id, device, texture));
	}
}

/*
pub struct CustomProgram<P: Pipeline> {
	state: Option<ProgramState<P>>,
}

impl<P: Pipeline> CustomProgram<P> {
	pub fn new() -> Self {
		Self { state: None }
	}
}

impl<P: Pipeline> Program for CustomProgram<P> {
	fn compile(&mut self, ctx: &mut RenderContext) {
		let device = ctx.device;
		let pipeline = P::init(device);
		let uniform_alignment =
			device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

		let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: uniform_alignment,
			mapped_at_creation: false,
		});

		let actor_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Actor Buffer"),
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			size: MAX_OBJECTS * uniform_alignment,
			mapped_at_creation: false,
		});

		let camera_size = size_of::<CameraUniform>() as wgpu::BufferAddress;
		let actor_size = size_of::<ActorUniform>() as wgpu::BufferAddress;

		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Program Bind Group"),
			layout: pipeline.bind_group_layout(),
			entries: &[
				// Camera
				wgpu::BindGroupEntry {
					binding: CAMERA_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &camera_buffer,
						size: wgpu::BufferSize::new(camera_size),
						offset: 0,
					}),
				},
				// Actors
				wgpu::BindGroupEntry {
					binding: ACTOR_BINDING,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &actor_buffer,
						size: wgpu::BufferSize::new(actor_size),
						offset: 0,
					}),
				},
			],
		});

		self.state = Some(ProgramState {
			pipeline,
			bind_group,
			camera_buffer,
			actor_buffer,
		});
	}
}
*/
