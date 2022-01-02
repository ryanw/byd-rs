use std::{
	collections::HashMap,
	error::Error,
	num::NonZeroU32,
	sync::atomic::{AtomicUsize, Ordering},
};
use winit::dpi::PhysicalSize;
use winit::{event::WindowEvent, window::Window as WinitWindow};

use crate::{App, DrawContext};

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
	pub surface_config: Option<wgpu::SurfaceConfiguration>,
	pub surface: Option<wgpu::Surface>,

	// Terminal
	pub surface_texture: Option<wgpu::Texture>,
	pub surface_texture_view: Option<wgpu::TextureView>,
	pub surface_texture_size: Option<wgpu::Extent3d>,

	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
	pub size: winit::dpi::PhysicalSize<u32>,
	current_pipeline: Option<PipelineID>,
	pub(crate) pipelines: HashMap<PipelineID, wgpu::RenderPipeline>,
}

impl State {
	pub async fn new(window: Option<&WinitWindow>) -> Self {
		let instance = wgpu::Instance::new(wgpu::Backends::all());
		let surface = unsafe { window.map(|win| instance.create_surface(win)) };
		let adapter = instance
			.request_adapter(&wgpu::RequestAdapterOptions {
				power_preference: wgpu::PowerPreference::default(),
				compatible_surface: surface.as_ref(),
				force_fallback_adapter: false,
			})
			.await
			.expect("Failed to request adapter");

		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
					limits: wgpu::Limits::default(),
					label: None,
				},
				None, // Trace path
			)
			.await
			.expect("Failed to request device");

		let size = match window {
			Some(window) => window.inner_size(),
			// FIXME get term size
			None => PhysicalSize::new(128, 128),
		};

		let surface_config = surface.as_ref().map(|surface| {
			let config = wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: surface
					.get_preferred_format(&adapter)
					.expect("Failed to get preferred surface format"),
				width: size.width,
				height: size.height,
				present_mode: wgpu::PresentMode::Fifo,
			};
			surface.configure(&device, &config);
			config
		});

		let (surface_texture, surface_texture_view, surface_texture_size) = if surface.is_none() {
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
				usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
				label: None,
			};
			let surface_texture = device.create_texture(&surface_texture_desc);
			let surface_texture_view = surface_texture.create_view(&Default::default());
			(
				Some(surface_texture),
				Some(surface_texture_view),
				Some(surface_texture_desc.size),
			)
		} else {
			(None, None, None)
		};

		Self {
			surface,
			surface_config,
			surface_texture,
			surface_texture_view,
			surface_texture_size,
			device,
			queue,
			size,
			pipelines: HashMap::new(),
			current_pipeline: None,
		}
	}

	pub fn add_pipeline(&mut self, pipeline: wgpu::RenderPipeline) -> PipelineID {
		let id = NEXT_PIPELINE_ID.fetch_add(1, Ordering::Relaxed);
		self.pipelines.insert(id, pipeline);

		id
	}

	pub fn use_pipeline(&mut self, id: PipelineID) {
		self.current_pipeline = Some(id);
	}

	pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
		if new_size.width == 0 || new_size.height == 0 {
			return;
		}

		log::debug!("Resizing surface to: {:?}", new_size);
		self.size = new_size;

		if let Some(surface) = self.surface.as_ref() {
			if let Some(config) = self.surface_config.as_mut() {
				config.width = new_size.width;
				config.height = new_size.height;
				surface.configure(&self.device, config);
			}
		} else {
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
				usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
				label: None,
			};
			let surface_texture = self.device.create_texture(&surface_texture_desc);
			let surface_texture_view = surface_texture.create_view(&Default::default());
			self.surface_texture = Some(surface_texture);
			self.surface_texture_view = Some(surface_texture_view);
			self.surface_texture_size = Some(surface_texture_desc.size);
		}
	}

	pub fn input(&mut self, event: &WindowEvent) -> bool {
		false
	}

	pub fn update(&mut self) {
		// remove `todo!()`
	}

	pub fn render<A: App>(&mut self, app: &mut A) -> Result<(), Box<dyn Error>> {
		// FIXME don't unwrap
		let frame = self.surface.as_ref().unwrap().get_current_texture()?;
		let view = frame
			.texture
			.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});
		{
			let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: &view,
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

			let mut ctx = DrawContext::new(self, render_pass);
			app.draw(&mut ctx);
		}

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));
		frame.present();

		Ok(())
	}

	pub fn render_to_buffer<A: App>(
		&mut self,
		buffer: &mut wgpu::Buffer,
		app: &mut A,
	) -> Result<(), Box<dyn Error>> {
		let surface_texture_view = self.surface_texture_view.take();
		let tex_width = next_pow2(self.size.width);
		let tex_height = next_pow2(self.size.height);
		log::debug!(
			"Rendering to buffer: {}x{} => {}x{}",
			self.size.width,
			self.size.height,
			tex_width,
			tex_height
		);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("Render Encoder"),
			});
		{
			let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[wgpu::RenderPassColorAttachment {
					view: surface_texture_view.as_ref().unwrap(),
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

			let mut ctx = DrawContext::new(self, render_pass);
			app.draw(&mut ctx);
		}

		encoder.copy_texture_to_buffer(
			wgpu::ImageCopyTexture {
				texture: self.surface_texture.as_ref().unwrap(),
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			wgpu::ImageCopyBuffer {
				buffer: &buffer,
				layout: wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: NonZeroU32::new(4 * tex_width),
					rows_per_image: NonZeroU32::new(tex_height),
				},
			},
			self.surface_texture_size.unwrap(),
		);

		// submit will accept anything that implements IntoIter
		self.queue.submit(std::iter::once(encoder.finish()));

		self.surface_texture_view = surface_texture_view;

		Ok(())
	}
}
