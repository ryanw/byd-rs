use std::{
	collections::HashMap,
	num::NonZeroU32,
	sync::atomic::{AtomicUsize, Ordering},
};
use winit::dpi::PhysicalSize;
#[cfg(target_os = "linux")]
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{pipelines::QuadPipeline, App, DrawContext};

fn next_pow2(mut n: u32) -> u32 {
	if n <= 1 {
		return 1;
	}
	let mut p = 2;

	n -= 1;
	n >>= 1;
	while n != 0 {
		p <<= 1;
		n >>= 1;
	}

	p
}

pub type PipelineID = usize;
pub static NEXT_PIPELINE_ID: AtomicUsize = AtomicUsize::new(1);

pub struct State {
	// Screen
	pub surface: Option<wgpu::Surface>,

	// Rendered to texture
	pub surface_texture: wgpu::Texture,
	pub surface_texture_view: wgpu::TextureView,
	pub surface_texture_size: wgpu::Extent3d,

	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub sc_desc: Option<wgpu::SwapChainDescriptor>,
	pub swap_chain: Option<wgpu::SwapChain>,
	pub size: PhysicalSize<u32>,

	pub quad_pipeline: QuadPipeline,
}

impl State {
	pub async fn new(window: Option<&WinitWindow>) -> Self {
		let size = match window {
			Some(window) => window.inner_size(),
			// FIXME get term size
			None => PhysicalSize::new(128, 128),
		};

		let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
		let surface = window.map(|win| unsafe { instance.create_surface(win) });
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: surface.as_ref(),
			})
			.await
			.unwrap();

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					features: wgpu::Features::empty(),
					limits: wgpu::Limits::default(),
					label: None,
				},
				None, // Trace path
			)
			.await
			.unwrap();

		let (sc_desc, swap_chain) = match &surface {
			Some(surface) => {
				let sc_desc = wgpu::SwapChainDescriptor {
					usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
					format: adapter.get_swap_chain_preferred_format(surface).unwrap(),
					width: size.width,
					height: size.height,
					present_mode: wgpu::PresentMode::Fifo,
				};
				let swap_chain = device.create_swap_chain(surface, &sc_desc);
				(Some(sc_desc), Some(swap_chain))
			}
			None => (None, None),
		};

		let surface_texture_desc = wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: size.width,
				height: size.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			usage: wgpu::TextureUsage::COPY_SRC
				| wgpu::TextureUsage::RENDER_ATTACHMENT
				| wgpu::TextureUsage::SAMPLED,
			label: None,
		};
		let surface_texture = device.create_texture(&surface_texture_desc);
		let surface_texture_view = surface_texture.create_view(&Default::default());

		Self {
			quad_pipeline: QuadPipeline::new(&device, &surface_texture_view),
			surface,
			surface_texture,
			surface_texture_view,
			surface_texture_size: surface_texture_desc.size,
			device,
			queue,
			sc_desc,
			swap_chain,
			size,
		}
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		self.size = new_size;
		if let Some(sc_desc) = &mut self.sc_desc {
			sc_desc.width = new_size.width;
			sc_desc.height = new_size.height;
			if let Some(swap_chain) = &mut self.swap_chain {
				if let Some(surface) = &self.surface {
					*swap_chain = self.device.create_swap_chain(surface, sc_desc);
				}
			}
		}

		let surface_texture_desc = wgpu::TextureDescriptor {
			size: wgpu::Extent3d {
				width: self.size.width,
				height: self.size.height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			usage: wgpu::TextureUsage::COPY_SRC
				| wgpu::TextureUsage::RENDER_ATTACHMENT
				| wgpu::TextureUsage::SAMPLED,
			label: None,
		};
		let surface_texture = self.device.create_texture(&surface_texture_desc);
		let surface_texture_view = surface_texture.create_view(&Default::default());
		self.surface_texture = surface_texture;
		self.surface_texture_view = surface_texture_view;
		self.surface_texture_size = surface_texture_desc.size;
		self.quad_pipeline
			.set_texture_view(&self.device, &self.surface_texture_view);
	}

	pub fn input(&mut self, event: &WindowEvent) -> bool {
		false
	}

	pub fn update(&mut self) {
		// remove `todo!()`
	}

	pub fn render_to_texture<A: App>(
		&mut self,
		encoder: &mut wgpu::CommandEncoder,
		app: &mut A,
	) -> Result<(), wgpu::SwapChainError> {
		let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[wgpu::RenderPassColorAttachment {
				view: &self.surface_texture_view,
				resolve_target: None,
				ops: wgpu::Operations {
					load: wgpu::LoadOp::Clear(wgpu::Color {
						r: 0.2,
						g: 0.1,
						b: 0.3,
						a: 1.0,
					}),
					store: true,
				},
			}],
			depth_stencil_attachment: None,
		});

		let mut ctx = DrawContext::new(&self.device, &mut self.queue, render_pass);
		app.draw(&mut ctx);

		Ok(())
	}

	pub fn render_texture_to_quad<'a>(
		&'a mut self,
		render_pass: &mut wgpu::RenderPass<'a>,
	) -> Result<(), wgpu::SwapChainError> {
		self.quad_pipeline.draw_quad(render_pass);
		Ok(())
	}

	pub fn render<A: App>(&mut self, app: &mut A) -> Result<(), wgpu::SwapChainError> {
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render To Screen Encoder"),
			});

		// Render scene to texture
		self.render_to_texture(&mut encoder, app)?;

		// FIXME don't unwrap
		let frame = self
			.swap_chain
			.as_ref()
			.unwrap()
			.get_current_frame()?
			.output;

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Quad Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &frame.view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color {
							r: 0.2,
							g: 0.1,
							b: 0.3,
							a: 1.0,
						}),
						store: true,
					},
				}],
				depth_stencil_attachment: None,
			});

			self.render_texture_to_quad(&mut render_pass)?;
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));

		Ok(())
	}

	pub fn render_to_buffer<A: App>(
		&mut self,
		buffer: &mut wgpu::Buffer,
		app: &mut A,
	) -> Result<(), wgpu::SwapChainError> {
		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render To Buffer Encoder"),
			});

		self.render_to_texture(&mut encoder, app)?;

		// FIXME Infer size from buffer?
		let tex_width = next_pow2(self.surface_texture_size.width);
		let tex_height = next_pow2(self.surface_texture_size.height);
		encoder.copy_texture_to_buffer(
			wgpu::ImageCopyTexture {
				texture: &self.surface_texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
			},
			wgpu::ImageCopyBuffer {
				buffer: &buffer,
				layout: wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: NonZeroU32::new(4 * tex_width),
					rows_per_image: NonZeroU32::new(tex_height),
				},
			},
			self.surface_texture_size,
		);

		self.queue.submit(std::iter::once(encoder.finish()));

		Ok(())
	}
}
