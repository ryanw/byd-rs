mod simple;
pub use simple::*;
mod primitive;
pub use primitive::PrimitivePipeline;
mod custom_pipeline;
pub use custom_pipeline::CustomPipeline;
mod line;
pub use line::LinePipeline;
mod quad;
pub use quad::*;
use std::mem::size_of_val;

pub trait Uniform {
	fn as_bytes(&self) -> &[u8] {
		unsafe {
			let ptr = self as *const Self as *const u8;
			let len = size_of_val(self);
			std::slice::from_raw_parts(ptr, len)
		}
	}
}
