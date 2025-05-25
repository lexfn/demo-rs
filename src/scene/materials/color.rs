use crate::math::Vec3;
use crate::render;
use crate::render::{MaterialBuilder, PositionUvNormalVertex};
use crate::render::Renderer;

use super::super::components::{Camera, Transform};
use super::uniforms::{Vec3Uniform, WorldViewProjUniform};

pub struct ColorMaterial {
    material: render::Material,
}

impl ColorMaterial {
    pub fn new(rr: &Renderer, shader: &wgpu::ShaderModule, color: Vec3, wireframe: bool) -> Self {
        let material = MaterialBuilder::new()
            .with_uniform_buffer(rr, WorldViewProjUniform::default())
            .with_uniform_buffer(rr, Vec3Uniform::new(color))
            .wireframe(wireframe)
            // TODO Leaner vertex format. Can't use it currently because this material
            // is used for file-loaded meshes where we currently only support a single vertex format.
            .build::<PositionUvNormalVertex>(rr, shader);

        Self { material }
    }
}

impl ColorMaterial {
    pub fn set_wvp(&self, rr: &Renderer, cam: &Camera, cam_tr: &Transform, tr: &Transform) {
        self.material.update_buffer(
            rr,
            0,
            WorldViewProjUniform::new(&tr.matrix(), &cam_tr.view_matrix(), &cam.proj_matrix()),
        );
    }

    pub fn apply<'a>(&'a self, encoder: &mut wgpu::RenderBundleEncoder<'a>) {
        self.material.apply(encoder);
    }
}
