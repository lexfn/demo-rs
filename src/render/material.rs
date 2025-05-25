use super::vertex::Vertex;
use super::{Renderer, Texture};

pub struct Material {
    pipeline: wgpu::RenderPipeline,
    bind_groups: Vec<wgpu::BindGroup>,
    uniform_bufs: Vec<wgpu::Buffer>,
}

pub struct MaterialBuilder {
    bind_groups: Vec<(wgpu::BindGroup, wgpu::BindGroupLayout)>,
    uniform_bufs: Vec<wgpu::Buffer>,
    wireframe: bool,
    depth_enabled: bool,
    depth_write: bool,
}

impl MaterialBuilder {
    pub fn new() -> Self {
        Self {
            bind_groups: Vec::new(),
            uniform_bufs: Vec::new(),
            wireframe: false,
            depth_enabled: true,
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

        let layout = rr.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        let pipeline = rr.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: Some("vs_main"),
                buffers: &[V::buffer_layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: rr.surface_texture_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: if self.wireframe {
                    wgpu::PrimitiveTopology::LineList
                } else {
                    wgpu::PrimitiveTopology::TriangleList
                },
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: if self.wireframe {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: if self.depth_enabled {
                Some(wgpu::DepthStencilState {
                    format: Renderer::DEPTH_TEX_FORMAT,
                    depth_write_enabled: self.depth_write,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
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
