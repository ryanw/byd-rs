use std::num::NonZeroU32;

use image::{ImageBuffer, Rgba};

use crate::RenderContext;

pub struct Texture {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl Texture {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

	pub fn new(device: &wgpu::Device, width: u32, height: u32, label: &str) -> Self {
		let label = format!("{} texture", label);
		let desc = wgpu::TextureDescriptor {
			label: Some(&label),
			size: wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING
				| wgpu::TextureUsages::COPY_SRC
				| wgpu::TextureUsages::COPY_DST
				| wgpu::TextureUsages::RENDER_ATTACHMENT,
		};
		let texture = device.create_texture(&desc);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some(&format!("{} sampler", label)),
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Nearest,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		Self {
			texture,
			view,
			sampler,
		}
	}

	pub fn new_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> Self {
		let desc = wgpu::TextureDescriptor {
			label: Some("Depth Texture"),
			size: wgpu::Extent3d {
				width,
				height,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
		};
		let texture = device.create_texture(&desc);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			compare: Some(wgpu::CompareFunction::LessEqual),
			lod_min_clamp: -100.0,
			lod_max_clamp: 100.0,
			..Default::default()
		});

		Self {
			texture,
			view,
			sampler,
		}
	}

	pub fn write(&self, queue: &mut wgpu::Queue, contents: &ImageBuffer<Rgba<u8>, Vec<u8>>) {
		log::debug!("Writing texture to GPU");
		let width = contents.width();
		let height = contents.height();
		let texture_size = wgpu::Extent3d {
			width,
			height,
			depth_or_array_layers: 1,
		};
		queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d::ZERO,
				aspect: wgpu::TextureAspect::All,
			},
			contents,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: NonZeroU32::new(4 * width),
				rows_per_image: NonZeroU32::new(height),
			},
			texture_size,
		);
	}
}
