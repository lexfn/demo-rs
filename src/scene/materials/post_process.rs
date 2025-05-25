use crate::render::Texture;
use crate::render::{PositionUvVertex, Vertex};
use crate::render::{RenderPipelineParams, Renderer};

pub struct PostProcessMaterial {
    pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
}

impl PostProcessMaterial {
    // TODO Passing shader here is weird because the material should dictate which shader to use.
    // Either avoid passing it or make the material generic and accept *any* shader.
    // Same for other materials.
    pub fn new(rr: &Renderer, shader: &wgpu::ShaderModule, texture: &Texture) -> Self {
        let (texture_bind_group_layout, texture_bind_group) =
            rr.new_texture_bind_group(texture, wgpu::TextureViewDimension::D2);

        let pipeline = rr.new_render_pipeline(RenderPipelineParams {
            shader_module: shader,
            depth_write: true,
            depth_enabled: true,
            wireframe: false,
            bind_group_layouts: &[&texture_bind_group_layout],
            vertex_buffer_layouts: &[PositionUvVertex::buffer_layout()],
        });

        Self {
            pipeline,
            texture_bind_group,
        }
    }

    pub fn apply<'a>(&'a self, encoder: &mut wgpu::RenderBundleEncoder<'a>) {
        encoder.set_pipeline(&self.pipeline);
        encoder.set_bind_group(0, &self.texture_bind_group, &[]);
    }
}
