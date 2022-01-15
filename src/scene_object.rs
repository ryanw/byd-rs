use crate::{BasicMaterial, Color, Material, MountContext, RenderContext};
use cgmath::{Matrix4, SquareMatrix};
use downcast_rs::{impl_downcast, Downcast};

pub static DEFAULT_MATERIAL: BasicMaterial = BasicMaterial::new(Color::new(1.0, 0.25, 0.1, 1.0));

pub trait SceneObject: Downcast {
	fn render<'a>(&'a mut self, _ctx: &mut RenderContext<'a>) {}
	fn mount(&mut self, _ctx: &mut MountContext) {}
	fn unmount(&mut self, _ctx: &mut MountContext) {}
	fn transform(&self) -> Matrix4<f32> {
		Matrix4::identity()
	}
	fn material(&self) -> &dyn Material {
		&DEFAULT_MATERIAL
	}
}
impl_downcast!(SceneObject);
