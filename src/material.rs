use crate::Color;

pub trait Material {}

#[derive(Clone)]
pub struct BasicMaterial {
	pub color: Color,
}

impl Material for BasicMaterial {}

impl BasicMaterial {
	pub fn new(color: Color) -> Self {
		Self { color }
	}
}
