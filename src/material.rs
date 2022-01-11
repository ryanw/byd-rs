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
	pub fn new(color: Color) -> Self {
		Self { color }
	}
}

#[derive(Clone)]
pub struct LineMaterial {}

impl Material for LineMaterial {}

impl LineMaterial {
	pub fn new() -> Self {
		Self {}
	}
}
