use crate::{AsUniformValue, UniformValue};
use std::{error::Error, ffi::CString, fmt, ptr};

#[derive(Debug)]
pub struct ShaderError(String);
impl Error for ShaderError {}

impl fmt::Display for ShaderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone)]
pub struct Pipeline {
	pipeline_id: u32,
	vertex_shader_id: u32,
	fragment_shader_id: u32,
	compute_shader_id: u32,
}

impl Pipeline {
	pub fn new() -> Self {
		Self {
			pipeline_id: 0,
			vertex_shader_id: 0,
			fragment_shader_id: 0,
			compute_shader_id: 0,
		}
	}

	pub fn bind(&self) {
		todo!();
	}

	pub fn uniform_location(&self, name: &str) -> i32 {
		todo!();
	}

	pub fn bind_uniform(&self, name: &str, value: &impl AsUniformValue) {
		todo!();
	}
}
