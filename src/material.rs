use crate::Color;
use downcast_rs::{impl_downcast, Downcast};

pub trait Material: Downcast {}
impl_downcast!(Material);

#[derive(Clone)]
pub struct BasicMaterial {
	pub color: Color,
}

impl Material for BasicMaterial {}

impl BasicMaterial {
	pub const fn new(color: Color) -> Self {
		Self { color }
	}
}

#[derive(Clone)]
pub struct LineMaterial {}

impl Material for LineMaterial {}

impl LineMaterial {
	pub const fn new() -> Self {
		Self {}
	}
}

#[derive(Clone)]
pub struct TextureMaterial {
	pub texture_id: usize,
}

impl Material for TextureMaterial {}

impl TextureMaterial {
	pub const fn new(texture_id: usize) -> Self {
		Self { texture_id }
	}
}
