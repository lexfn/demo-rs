mod material;
mod mesh;
mod render_target;
mod renderer;
mod texture;
mod ui;
mod vertex;

use crate::math::Mat4;
pub use material::{ApplyMaterial, Material, MaterialBuilder};
pub use mesh::Mesh;
pub use render_target::RenderTarget;
pub use renderer::{RenderPipelineParams, Renderer, SurfaceSize};
pub use texture::Texture;
pub use ui::Ui;
pub use vertex::{PositionUvNormalVertex, PositionUvVertex};

// Converts from "OpenGL format" to WGPU.
#[rustfmt::skip]
pub const WGPU_CONVERSION_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);
