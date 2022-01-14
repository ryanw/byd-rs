use byd::{BasicMaterial, Color, Geometry, Mesh, SimpleVertex};
use cgmath::{InnerSpace, Point2, Point3, Vector3};
use noise::{Fbm, MultiFractal, NoiseFn, Seedable};

pub struct Terrain {
	noise: Fbm,
}

impl Terrain {
	pub fn new() -> Self {
		let noise = Fbm::new().set_octaves(3).set_seed(666);

		Self { noise }
	}

	pub fn generate_mesh(&self, x_offset: u32, z_offset: u32) -> Mesh<SimpleVertex, BasicMaterial> {
		let mut vertices = vec![];
		let width = 32;
		let depth = 32;
		let scale = 0.08;

		for z in -depth..=depth {
			for x in -width..=width {
				for p in &QUAD_POINTS {
					let px = scale * (p[0] + x as f32 + x_offset as f32);
					let py = scale * (p[2] + z as f32 + z_offset as f32);

					// Position
					let point = Point2::new(px, py);
					let y = self.height(&point);

					let position = Point3::new(
						p[0] + x as f32 + x_offset as f32,
						p[1] + y,
						p[2] + z as f32 + z_offset as f32,
					);

					// Normal
					let off = Vector3::new(0.08, 0.08, 0.0);
					let hl = self.height(&Point2::new(point.x - off.x, point.y - off.z));
					let hr = self.height(&Point2::new(point.x + off.x, point.y + off.z));
					let hd = self.height(&Point2::new(point.x - off.z, point.y - off.y));
					let hu = self.height(&Point2::new(point.x + off.z, point.y + off.y));
					let normal = Vector3::new(hl - hr, 2.0, hd - hu).normalize();

					vertices.push(SimpleVertex {
						position,
						normal,
						..Default::default()
					});
				}
			}
		}

		let mesh = Mesh::new(
			Geometry::new(vertices),
			BasicMaterial::new(Color::new(0.0, 0.5, 0.1, 1.0)),
		);

		mesh
	}

	fn height(&self, pos: &Point2<f32>) -> f32 {
		self.noise.get([pos.x as f64, pos.y as f64, 0.0]) as f32 * 4.0
	}

	fn normal(&self, pos: &Point2<f32>) -> f32 {
		self.noise.get([pos.x as f64, pos.y as f64, 0.0]) as f32 * 4.0
	}
}

fn calculate_normal(tri: &[SimpleVertex]) -> Vector3<f32> {
	let u = tri[1].position - tri[0].position;
	let v = tri[2].position - tri[0].position;

	u.cross(v)
}

const QUAD_POINTS: [[f32; 3]; 6] = [
	[-0.5, 0.0, -0.5],
	[-0.5, 0.0, 0.5],
	[0.5, 0.0, -0.5],
	[0.5, 0.0, 0.5],
	[0.5, 0.0, -0.5],
	[-0.5, 0.0, 0.5],
];
