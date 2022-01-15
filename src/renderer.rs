use crate::{
	pipelines::{QuadPipeline, Vertex as QuadVertex},
	Camera, Pipeline, RenderContext, Scene, TextureBuffer, Window,
};
use std::{
	error::Error,
	ops::{Deref, DerefMut},
};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

pub struct Renderer {
	surface: Option<wgpu::Surface>,
	adapter: wgpu::Adapter,
	instance: wgpu::Instance,
	size: wgpu::Extent3d,
	device: wgpu::Device,
	queue: wgpu::Queue,
	quad: Quad,

	depth_texture: TextureBuffer,
	screen_texture: TextureBuffer,
}

struct Quad {
	buffer: wgpu::Buffer,
	pipeline: QuadPipeline,
	bind_group: wgpu::BindGroup,
}

impl Renderer {
	pub async fn new(width: u32, height: u32) -> Self {
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: None,
				force_fallback_adapter: false,
			})
			.await
			.expect("Failed to request adapter");
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: Some("Device Descriptor"),
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
				},
				None, // Trace path
			)
			.await
			.expect("Failed to request device");

		let depth_texture = TextureBuffer::new_depth_texture(&device, width, height);
		let screen_texture = TextureBuffer::new(&device, width, height, "Screen");

		let quad = Quad::new(&device, &screen_texture);

		let size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};

		Self {
			surface: None,
			quad,
			adapter,
			instance,
			size,
			device,
			queue,
			depth_texture,
			screen_texture,
		}
	}

	pub fn attach(&mut self, window: &Window) {
		let surface = unsafe { self.instance.create_surface(&window.winit) };
		self.surface = Some(surface);
		self.resize(self.size.width, self.size.height);
	}

	pub fn resize(&mut self, width: u32, height: u32) {
		self.size.width = width;
		self.size.height = height;

		if let Some(surface) = self.surface.as_ref() {
			log::debug!("Resizing renderer surface {}x{}", width, height);
			let config = wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: surface
					.get_preferred_format(&self.adapter)
					.expect("Failed to get preferred surface format"),
				width: self.size.width,
				height: self.size.height,
				present_mode: wgpu::PresentMode::Fifo,
			};
			surface.configure(&self.device, &config);
		}

		log::debug!("Resizing renderer texture {}x{}", width, height);
		self.screen_texture = TextureBuffer::new(&self.device, width, height, "Screen");
		self.quad.set_texture(&self.device, &self.screen_texture);

		log::debug!("Resizing depth texture");
		self.depth_texture = TextureBuffer::new_depth_texture(&self.device, width, height);
	}

	pub fn render<SR, CR, C>(&mut self, mut scene: SR, camera: CR) -> Result<(), Box<dyn Error>>
	where
		SR: DerefMut<Target = Scene>,
		CR: Deref<Target = C>,
		C: Camera,
	{
		self.render_to_buffer(&mut *scene, &*camera)?;
		self.render_to_surface()?;

		Ok(())
	}

	pub fn render_to_buffer(
		&mut self,
		scene: &mut Scene,
		camera: &dyn Camera,
	) -> Result<(), Box<dyn Error>> {
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});
		{
			let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &self.screen_texture.view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.05,
							g: 0.05,
							b: 0.05,
							a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture.view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: true,
					}),
					stencil_ops: None,
				}),
			});

			scene.process_texture_queue(&self.device, &mut self.queue);

			// Draw everything
			let mut ctx = RenderContext {
				device: &self.device,
				queue: &mut self.queue,
				render_pass,
				camera,
			};
			scene.render(&mut ctx);
		}
		self.queue.submit(std::iter::once(encoder.finish()));

		Ok(())
	}

	pub fn render_to_surface(&mut self) -> Result<(), Box<dyn Error>> {
		if let Some(surface) = &self.surface {
			let frame = surface.get_current_texture()?;
			let view = frame
				.texture
				.create_view(&wgpu::TextureViewDescriptor::default());

			let mut encoder = self
				.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("Quad Render Encoder"),
				});
			{
				let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: Some("Quad Render Pass"),
					color_attachments: &[wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color {
								r: 0.1,
								g: 0.1,
								b: 0.3,
								a: 1.0,
							}),
							store: true,
						},
					}],
					depth_stencil_attachment: None,
				});

				// Draw the quad
				self.quad.render(&mut render_pass);
			}

			// submit will accept anything that implements IntoIter
			self.queue.submit(std::iter::once(encoder.finish()));
			frame.present();
		}

		Ok(())
	}
}

impl Quad {
	fn new(device: &wgpu::Device, texture: &TextureBuffer) -> Self {
		let pipeline = QuadPipeline::new(device);
		let buffer = device.create_buffer_init(&BufferInitDescriptor {
			label: Some("Quad Vertex Buffer"),
			usage: wgpu::BufferUsages::VERTEX,
			contents: bytemuck::cast_slice(&QUAD_VERTICES),
		});
		let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("QuadPipeline Bind Group"),
			layout: pipeline.bind_group_layout(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
		});

		Self {
			pipeline,
			buffer,
			bind_group,
		}
	}

	fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
		self.pipeline.apply(render_pass);
		render_pass.set_bind_group(0, &self.bind_group, &[]);
		render_pass.set_vertex_buffer(0, self.buffer.slice(..));
		render_pass.draw(0..QUAD_VERTICES.len() as _, 0..1);
	}

	fn set_texture(&mut self, device: &wgpu::Device, texture: &TextureBuffer) {
		self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("QuadPipeline Bind Group"),
			layout: self.pipeline.bind_group_layout(),
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&texture.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&texture.sampler),
				},
			],
		});
	}
}

#[cfg_attr(rustfmt, rustfmt_skip)]
const QUAD_VERTICES: [QuadVertex; 6] = [
	QuadVertex::new( 1.0,  1.0, 0.5),
	QuadVertex::new(-1.0,  1.0, 0.5),
	QuadVertex::new( 1.0, -1.0, 0.5),
	QuadVertex::new(-1.0, -1.0, 0.5),
	QuadVertex::new( 1.0, -1.0, 0.5),
	QuadVertex::new(-1.0,  1.0, 0.5),
];
