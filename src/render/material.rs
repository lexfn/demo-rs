use super::vertex::Vertex;
use super::{RenderPipelineParams, Renderer, Texture};

pub struct Material {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_bufs: Vec<wgpu::Buffer>,
}

pub struct MaterialBuilder {
    bind_groups: Vec<(wgpu::BindGroup, wgpu::BindGroupLayout)>,
    uniform_bufs: Vec<wgpu::Buffer>,
    wireframe: bool,
    depth_write: bool,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            bind_groups: Vec::new(),
            uniform_bufs: Vec::new(),
            wireframe: false,
            depth_write: true,
        }
    }

    pub fn with_2d_texture(self, rr: &Renderer, texture: &Texture) -> Self {
        let (bind_group_layout, bind_group) =
            rr.new_texture_bind_group(texture, wgpu::TextureViewDimension::D2);
        Self {
            bind_groups: self
                .bind_groups
                .into_iter()
                .chain([(bind_group, bind_group_layout)])
                .collect(),
            ..self
        }
    }

    pub fn with_cube_texture(self, rr: &Renderer, texture: &Texture) -> Self {
        let (bind_group_layout, bind_group) =
            rr.new_texture_bind_group(texture, wgpu::TextureViewDimension::Cube);
        Self {
            bind_groups: self
                .bind_groups
                .into_iter()
                .chain([(bind_group, bind_group_layout)])
                .collect(),
            ..self
        }
    }

    pub fn with_uniform(self, rr: &Renderer, uniform: impl bytemuck::NoUninit) -> Self {
        let (bind_group_layout, bind_group, buf) =
            rr.new_uniform_bind_group(bytemuck::cast_slice(&[uniform]));

        Self {
            bind_groups: self
                .bind_groups
                .into_iter()
                .chain([(bind_group, bind_group_layout)])
                .collect(),
            uniform_bufs: self.uniform_bufs.into_iter().chain([buf]).collect(),
            ..self
        }
    }

    pub fn wireframe(self, wireframe: bool) -> Self {
        Self { wireframe, ..self }
    }

    pub fn depth_write(self, write: bool) -> Self {
        Self {
            depth_write: write,
            ..self
        }
    }

    pub fn build<V: Vertex>(self, rr: &Renderer, shader: &wgpu::ShaderModule) -> Material {
        let (bind_groups, bind_group_layouts): (Vec<_>, Vec<_>) =
            self.bind_groups.into_iter().unzip();

        let pipeline = rr.new_render_pipeline(RenderPipelineParams {
            shader_module: shader,
            depth_write: self.depth_write,
            depth_enabled: true,
            wireframe: self.wireframe,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            vertex_buffer_layouts: &[V::buffer_layout()],
        });

        Material {
            pipeline,
            bind_groups,
            uniform_bufs: self.uniform_bufs,
        }
    }
}

impl Material {
    pub fn update_buffer(&self, rr: &Renderer, buf_idx: usize, data: impl bytemuck::NoUninit) {
        rr.queue().write_buffer(
            &self.uniform_bufs[buf_idx],
            0,
            bytemuck::cast_slice(&[data]),
        );
    }

    pub fn apply<'a>(&'a self, encoder: &mut wgpu::RenderBundleEncoder<'a>) {
        encoder.set_pipeline(&self.pipeline);
        for (idx, group) in self.bind_groups.iter().enumerate() {
            encoder.set_bind_group(idx as _, group, &[]);
        }
    }
}

// TODO Try to remove
pub trait ApplyMaterial {
    fn apply<'a>(&'a self, encoder: &mut wgpu::RenderBundleEncoder<'a>);
}
