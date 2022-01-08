pub type ActorID = usize;

pub mod pipelines;

pub mod camera;
pub use camera::*;

pub mod render_context;
pub use render_context::*;

pub mod mount_context;
pub use mount_context::*;

pub mod material;
pub use material::*;

pub mod geometry;
pub use geometry::*;

pub mod mesh;
pub use mesh::*;

pub mod uniforms;
pub use uniforms::*;

pub mod event;
pub use event::*;

pub mod scene;
pub use scene::*;

pub mod window;
pub use window::*;

pub mod renderer;
pub use renderer::*;

pub mod vertex;
pub use vertex::*;

/*
#[cfg(unix)]
pub mod term;
#[cfg(unix)]
pub use term::*;
*/

pub type Color = u32;
