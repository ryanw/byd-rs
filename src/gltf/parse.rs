use serde::Deserialize;
use std::{
	collections::HashMap,
	fs::File,
	io::{Read, Seek, SeekFrom},
	mem,
	ops::Range,
	path::Path,
};

#[derive(Deserialize, Debug)]
pub struct GltfDoc {
	#[serde(skip)]
	pub uri: String,
	pub asset: AssetDoc,
	#[serde(default, rename(deserialize = "extensionsUsed"))]
	pub extensions_used: Vec<String>,
	#[serde(default, rename(deserialize = "extensionsRequired"))]
	pub extensions_required: Vec<String>,
	pub scene: u64,
	pub scenes: Vec<SceneDoc>,
	pub nodes: Vec<NodeDoc>,
	pub materials: Vec<MaterialDoc>,
	pub meshes: Vec<MeshDoc>,
	pub textures: Vec<TextureDoc>,
	pub images: Vec<ImageDoc>,
	pub accessors: Vec<AccessorDoc>,
	#[serde(rename(deserialize = "bufferViews"))]
	pub buffer_views: Vec<BufferViewDoc>,
	pub samplers: Vec<SamplerDoc>,
	pub buffers: Vec<BufferDoc>,
}

impl GltfDoc {
	pub fn accessor<T>(&self, index: u64) -> Vec<T> {
		let accessor = &self.accessors[index as usize];
		let view = &self.buffer_views[accessor.buffer_view as usize];
		let size = mem::size_of::<T>();
		let mut bytes = self.read_buffer_range(
			view.buffer,
			(accessor.byte_offset + view.byte_offset)
				..(accessor.byte_offset + view.byte_offset + view.byte_length),
			view.byte_stride.unwrap_or(size as u64),
		);

		assert!(bytes.len() % size == 0);

		let p = bytes.as_mut_ptr();
		let len = bytes.len() / size;
		let cap = bytes.capacity() / size;
		mem::forget(bytes);
		unsafe { Vec::from_raw_parts(p as *mut T, len, cap) }
	}

	pub fn relative_filename(&self, filename: &str) -> String {
		let cwd = Path::new(&self.uri).parent().unwrap();
		cwd.join(filename).to_str().unwrap().into()
	}

	pub fn read_buffer_range(&self, index: u64, range: Range<u64>, stride: u64) -> Vec<u8> {
		// FIXME use stride
		let buffer = &self.buffers[index as usize];
		let filename = self.relative_filename(&buffer.uri);
		let mut file =
			File::open(filename).expect(&format!("Failed to open buffer: {}", buffer.uri));

		let mut data = vec![0; (range.end - range.start) as usize];
		file.seek(SeekFrom::Start(range.start)).expect(&format!(
			"Failed to seek to {} in {}",
			range.start, buffer.uri
		));
		file.read_exact(&mut data).expect(&format!(
			"Failed to read from {} to {} in {}",
			range.start, range.end, buffer.uri
		));

		data.shrink_to_fit();
		data
	}
}

#[derive(Deserialize, Debug)]
pub struct AssetDoc {
	#[serde(default)]
	pub generator: String,
	#[serde(default)]
	pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct SceneDoc {
	#[serde(default)]
	pub name: String,
	pub nodes: Vec<u64>,
}

#[derive(Deserialize, Debug)]
pub struct NodeDoc {
	#[serde(default)]
	pub name: String,
	pub mesh: Option<u64>,
	pub children: Option<Vec<u64>>,
	pub translation: Option<(f32, f32, f32)>,
	pub matrix: Option<[f32; 16]>,
}

#[derive(Deserialize, Debug)]
pub struct MaterialDoc {
	#[serde(default)]
	pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct MeshDoc {
	#[serde(default)]
	pub name: String,
	pub primitives: Vec<PrimitiveDoc>,
}

#[derive(Deserialize, Debug)]
pub struct TextureDoc {
	pub sampler: u64,
	pub source: u64,
}

#[derive(Deserialize, Debug)]
pub struct ImageDoc {
	#[serde(default, rename(deserialize = "mimeType"))]
	pub mime_type: String,
	#[serde(default)]
	pub name: String,
	pub uri: String,
}

#[derive(Deserialize, Debug)]
pub struct PrimitiveDoc {
	pub attributes: HashMap<String, u64>,
	pub indices: Option<u64>,
	pub material: u64,
}

#[derive(Deserialize, Debug)]
pub struct AccessorDoc {
	#[serde(rename(deserialize = "bufferView"))]
	pub buffer_view: u64,
	#[serde(default, rename(deserialize = "byteOffset"))]
	pub byte_offset: u64,
	#[serde(rename(deserialize = "componentType"))]
	pub component_type: u64,
	pub count: u64,
	#[serde(rename(deserialize = "type"))]
	pub ty: String,
}

#[derive(Deserialize, Debug)]
pub struct BufferViewDoc {
	pub buffer: u64,
	#[serde(rename(deserialize = "byteLength"))]
	pub byte_length: u64,
	#[serde(default, rename(deserialize = "byteOffset"))]
	pub byte_offset: u64,
	#[serde(rename(deserialize = "byteStride"))]
	pub byte_stride: Option<u64>,
	pub target: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct SamplerDoc {
	#[serde(rename(deserialize = "magFilter"))]
	pub mag_filter: u64,
	#[serde(rename(deserialize = "minFilter"))]
	pub min_filter: u64,
}

#[derive(Deserialize, Debug)]
pub struct BufferDoc {
	#[serde(rename(deserialize = "byteLength"))]
	pub byte_length: u64,
	#[serde(rename(deserialize = "type"))]
	pub ty: Option<String>,
	pub uri: String,
}
