use super::super::components::{Camera, Transform};
use super::super::Assets;
use super::uniforms::{Vec3Uniform, ViewInvProjUniform, WorldViewProjUniform};
use crate::math::Vec3;
use crate::render;
use crate::render::{MaterialBuilder, PositionUvNormalVertex, PositionUvVertex, Renderer, Texture};

pub enum Material {
    Color(render::Material),
    Textured(render::Material),
    Skybox(render::Material),
    PostProcess(render::Material),
}

impl Material {
    pub fn textured(rr: &Renderer, assets: &mut Assets, tex_path: &str) -> Self {
        let shader = assets.add_shader_from_file(rr, "textured.wgsl");
        let tex = assets.add_2d_texture_from_file(rr, tex_path);
        let material = MaterialBuilder::new()
            .with_uniform(rr, WorldViewProjUniform::default())
            // TODO We shouldn't call assets again to get the actual objects, they should be returned
            // from the Assets' methods that created them.
            .with_2d_texture(rr, assets.texture(tex))
            .build::<PositionUvNormalVertex>(rr, assets.shader(shader));
        Self::Textured(material)
    }

    pub fn post_process(rr: &Renderer, assets: &mut Assets, src_texture: &Texture) -> Self {
        let shader = assets.add_shader_from_file(rr, "post-process.wgsl");
        let material = MaterialBuilder::new()
            .with_2d_texture(rr, src_texture)
            // TODO We shouldn't call assets again to get the actual objects, they should be returned
            // from the Assets' methods that created them.
            .build::<PositionUvVertex>(rr, assets.shader(shader));
        Self::PostProcess(material)
    }

    pub fn skybox(rr: &Renderer, assets: &mut Assets, tex_path: &str) -> Self {
        let shader = assets.add_shader_from_file(rr, "skybox.wgsl");
        let tex = assets.add_cube_texture_from_file(rr, tex_path);
        let material = MaterialBuilder::new()
            .with_uniform(rr, ViewInvProjUniform::default())
            // TODO We shouldn't call assets again to get the actual objects, they should be returned
            // from the Assets' methods that created them.
            .with_cube_texture(rr, assets.texture(tex))
            .depth_write(false)
            .build::<PositionUvVertex>(rr, assets.shader(shader));
        Self::Skybox(material)
    }

    pub fn color(rr: &Renderer, assets: &mut Assets, color: Vec3, wireframe: bool) -> Self {
        let shader = assets.add_shader_from_file(rr, "color.wgsl");
        let material = MaterialBuilder::new()
            .with_uniform(rr, WorldViewProjUniform::default())
            .with_uniform(rr, Vec3Uniform::new(color))
            .wireframe(wireframe)
            // TODO Leaner vertex format. Can't use it currently because this material
            // is used for file-loaded meshes where we currently only support a single vertex format.
            // TODO We shouldn't call assets again to get the actual objects, they should be returned
            // from the Assets' methods that created them.
            .build::<PositionUvNormalVertex>(rr, assets.shader(shader));
        Self::Color(material)
    }

    pub fn update(&self, rr: &Renderer, cam: &Camera, cam_tr: &Transform, tr: &Transform) {
        match self {
            Material::Color(m) => m.update_buffer(
                rr,
                0,
                WorldViewProjUniform::new(&tr.matrix(), &cam_tr.view_matrix(), &cam.proj_matrix()),
            ),
            Material::Textured(m) => m.update_buffer(
                rr,
                0,
                WorldViewProjUniform::new(&tr.matrix(), &cam_tr.view_matrix(), &cam.proj_matrix()),
            ),
            Material::Skybox(m) => m.update_buffer(
                rr,
                0,
                ViewInvProjUniform::new(&cam_tr.view_matrix(), &cam.proj_matrix()),
            ),
            Material::PostProcess(_) => (),
        }
    }

    pub fn inner(&self) -> &render::Material {
        match self {
            Material::Color(m) => m,
            Material::Textured(m) => m,
            Material::Skybox(m) => m,
            Material::PostProcess(m) => m,
        }
    }
}
