use image::{
	io::Reader as ImageReader, DynamicImage, GenericImageView, ImageBuffer, ImageError, Rgba,
};
use std::{error, num::NonZeroU32};

pub struct Texture {
	width: u32,
	height: u32,
	pixels: ImageBuffer<Rgba<u8>, Vec<u8>>,
	buffer: Option<TextureBuffer>,
}

pub struct TextureBuffer {
	pub texture: wgpu::Texture,
	pub view: wgpu::TextureView,
	pub sampler: wgpu::Sampler,
}

impl Texture {
	pub fn from_image_bytes(bytes: &[u8]) -> Result<Self, ImageError> {
		let img = image::load_from_memory(bytes)?;

		Ok(Self::from_image(img))
	}

	pub fn from_image(img: DynamicImage) -> Self {
		let img = img.to_rgba8();
		let width = img.width();
		let height = img.height();

		Self {
			width,
			height,
			pixels: img,
			buffer: None,
		}
	}

	pub fn load(filename: &str) -> Result<Self, Box<dyn error::Error>> {
		log::debug!("Opening image file: {}", filename);
		let img = ImageReader::open(filename)?.decode()?;
		Ok(Self::from_image(img))
	}

	pub fn new(width: u32, height: u32) -> Self {
		Self {
			width,
			height,
			pixels: ImageBuffer::new(width, height),
			buffer: None,
		}
	}

	pub fn is_allocated(&self) -> bool {
		self.buffer.is_some()
	}

	pub fn allocate(&mut self, device: &wgpu::Device, label: &str) {
		self.destroy();
		self.buffer = Some(TextureBuffer::new(device, self.width, self.height, label));
	}

	pub fn destroy(&mut self) {
		self.buffer = None;
	}

	pub fn upload(&self, queue: &mut wgpu::Queue) {
		if let Some(buffer) = self.buffer.as_ref() {
			buffer.write(queue, &self.pixels);
		}
	}

	pub fn write(&self, queue: &mut wgpu::Queue, contents: &ImageBuffer<Rgba<u8>, Vec<u8>>) {
		// FIXME Error if there's no buffer -- or queue the request?
		if let Some(buffer) = self.buffer.as_ref() {
			buffer.write(queue, contents);
		}
	}

	/// Get a reference to the texture's buffer.
	pub fn buffer(&self) -> Option<&TextureBuffer> {
		self.buffer.as_ref()
	}
}

impl TextureBuffer {
	pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

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
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::TEXTURE_BINDING
				| wgpu::TextureUsages::COPY_SRC
				| wgpu::TextureUsages::COPY_DST
				| wgpu::TextureUsages::RENDER_ATTACHMENT,
		};
		let texture = device.create_texture(&desc);

		let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some(&format!("{} sampler", label)),
			address_mode_u: wgpu::AddressMode::Repeat,
			address_mode_v: wgpu::AddressMode::Repeat,
			address_mode_w: wgpu::AddressMode::Repeat,
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
