use crate::{BasicMaterial, Color, Geometry, Mesh, Texture, Vertex};
use byd_derive::CastBytes;
use cgmath::{Matrix4, Point2, Point3, SquareMatrix, Vector3, Vector4};
use std::{collections::HashMap, error, fs::File, io::BufReader, mem, path::Path};
use thiserror::Error;
use wgpu::VertexFormat::{Float32x2, Float32x3};

pub mod parse;
use parse::*;

#[derive(Error, Debug)]
pub enum GltfError {
	#[error("File not found")]
	FileNotFound,
	#[error("Unknown error")]
	Unknown(String),
}

impl From<Box<dyn error::Error>> for GltfError {
	fn from(error: Box<dyn error::Error>) -> Self {
		GltfError::Unknown(format!("{:?}", error))
	}
}

fn parse_gltf_json(filename: &str) -> Result<GltfDoc, Box<dyn error::Error>> {
	log::debug!("Reading glTF file: {}", filename);
	let file = File::open(filename)?;
	let reader = BufReader::new(file);
	let mut doc: GltfDoc = serde_json::from_reader(reader)?;
	doc.uri = filename.into();
	Ok(doc)
}

pub struct Gltf {
	pub meshes: Vec<Mesh<PrimitiveVertex>>,
	pub textures: Vec<Texture>,
	pub mesh_textures: HashMap<usize, usize>,
}

impl Gltf {
	pub fn load(filename: &str) -> Result<Self, GltfError> {
		let doc = parse_gltf_json(filename)?;

		let mut mesh_textures = HashMap::new();
		let mut meshes = vec![];
		let mut textures = vec![];

		fn load_meshes(
			mesh_id: u64,
			doc: &GltfDoc,
			meshes: &mut Vec<Mesh<PrimitiveVertex>>,
			mesh_textures: &mut HashMap<usize, usize>,
			transform: Matrix4<f32>,
		) {
			let mesh = &doc.meshes[mesh_id as usize];
			for PrimitiveDoc {
				indices: indices_id,
				attributes,
				material,
			} in &mesh.primitives
			{
				let positions_id = *attributes.get("POSITION").unwrap();
				let positions: Vec<Point3<f32>> = doc.accessor(positions_id);

				let normals: Vec<Vector3<f32>> = if let Some(normals_id) = attributes.get("NORMAL")
				{
					doc.accessor(*normals_id)
				} else {
					vec![]
				};

				let texcoords: Vec<Point2<f32>> =
					if let Some(texcoords_id) = attributes.get("TEXCOORD_0") {
						doc.accessor(*texcoords_id)
					} else {
						vec![]
					};

				let indices: Vec<u16> = if let Some(indices_id) = indices_id {
					doc.accessor(*indices_id)
				} else {
					(0..positions.len() as u16).collect()
				};

				let mut vertices = Vec::with_capacity(indices.len());
				for index in indices {
					vertices.push(PrimitiveVertex {
						position: positions[index as usize].clone(),
						normal: normals
							.get(index as usize)
							.unwrap_or(&Vector3::new(0.0, 0.0, 0.0))
							.clone(),
						texcoord: texcoords
							.get(index as usize)
							.unwrap_or(&Point2::new(0.0, 0.0))
							.clone(),
						..Default::default()
					});
				}

				let mut mesh = Mesh::new(
					Geometry::new(vertices),
					BasicMaterial::new(Color::new(
						rand::random::<f32>(),
						rand::random::<f32>(),
						rand::random::<f32>(),
						1.0,
					)),
				);
				mesh.transform = transform;
				meshes.push(mesh);

				let texture_id = doc.textures[*material as usize].source as usize;
				mesh_textures.insert(meshes.len() - 1, texture_id);
			}
		}

		fn load_node(
			node_id: u64,
			doc: &GltfDoc,
			meshes: &mut Vec<Mesh<PrimitiveVertex>>,
			mesh_textures: &mut HashMap<usize, usize>,
			mut transform: Matrix4<f32>,
		) {
			let node = &doc.nodes[node_id as usize];
			let mat: Matrix4<f32> = if let Some(node_trans) = node.translation.as_ref() {
				Matrix4::from_translation(Vector3::from(*node_trans))
			} else if let Some(node_mat) = node.matrix.as_ref() {
				// FIXME hax
				let cols = unsafe { mem::transmute::<[f32; 16], [[f32; 4]; 4]>(*node_mat) };
				Matrix4::from(cols)
			} else {
				Matrix4::identity()
			};
			transform = transform * mat;
			if let Some(mesh_id) = node.mesh {
				load_meshes(mesh_id, doc, meshes, mesh_textures, transform.clone());
			}
			if let Some(children) = node.children.as_ref() {
				for child_id in children {
					load_node(*child_id, doc, meshes, mesh_textures, transform.clone());
				}
			}
		}

		for scene in &doc.scenes {
			for node_id in &scene.nodes {
				load_node(
					*node_id,
					&doc,
					&mut meshes,
					&mut mesh_textures,
					Matrix4::identity(),
				);
			}
		}

		for image in &doc.images {
			let filename = Path::new(filename)
				.parent()
				.unwrap()
				.join(&image.uri)
				.to_str()
				.unwrap()
				.to_string();

			textures.push(
				Texture::load(&filename).expect(&format!("Failed to open image: {}", filename)),
			);
		}

		Ok(Self {
			meshes,
			textures,
			mesh_textures,
		})
	}
}

#[derive(Copy, Clone, Debug, CastBytes)]
#[repr(C)]
pub struct PrimitiveVertex {
	position: Point3<f32>,
	normal: Vector3<f32>,
	texcoord: Point2<f32>,
	//color: Color,
	// TODO support multiple texcoords and colours
}

impl Default for PrimitiveVertex {
	fn default() -> Self {
		Self {
			position: Point3::new(0.0, 0.0, 0.0),
			normal: Vector3::new(0.0, 0.0, 0.0),
			texcoord: Point2::new(0.0, 0.0),
			//color: Color::new(1.0, 0.0, 0.0, 1.0),
		}
	}
}

impl Vertex for PrimitiveVertex {
	fn buffer_layout<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<Self>() as _,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: mem::size_of::<Point3<f32>>() as _,
					shader_location: 1,
					format: Float32x3,
				},
				wgpu::VertexAttribute {
					offset: (mem::size_of::<Point3<f32>>() + mem::size_of::<Vector3<f32>>()) as _,
					shader_location: 2,
					format: Float32x2,
				},
			],
		}
	}
}

fn calculate_normal(tri: &[PrimitiveVertex]) -> Vector3<f32> {
	let u = tri[1].position - tri[0].position;
	let v = tri[2].position - tri[0].position;

	u.cross(v)
}
