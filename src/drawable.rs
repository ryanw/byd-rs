use crate::context::{DrawContext, MountContext};
use downcast_rs::{impl_downcast, Downcast};
use std::sync::{Arc, RwLock};

pub trait Drawable: Downcast {
	fn draw<'a>(&'a mut self, ctx: &mut DrawContext<'a>);
	fn mount(&mut self, _ctx: &mut MountContext) {}
	fn unmount(&mut self, _ctx: &mut MountContext) {}
}

impl_downcast!(Drawable);
