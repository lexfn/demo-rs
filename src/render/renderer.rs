use std::ops::Deref;
use std::sync::Arc;
use wgpu::util::DeviceExt;

use super::material::ApplyMaterial;
use super::mesh::Mesh;
use super::render_target::RenderTarget;
use super::texture::Texture;
use super::ui::Ui;

pub type SurfaceSize = winit::dpi::PhysicalSize<u32>;

pub struct Renderer<'a> {
    pub adapter_name: String,
    surface: wgpu::Surface<'a>,
    surface_cfg: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_tex: Texture,
}

impl<'a> Renderer<'a> {
    // TODO Configurable?
    pub const DEPTH_TEX_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn surface_texture_format(&self) -> wgpu::TextureFormat {
        self.surface_cfg.format
    }

    pub fn depth_texture_format(&self) -> wgpu::TextureFormat {
        Self::DEPTH_TEX_FORMAT
    }

    pub fn surface_size(&self) -> SurfaceSize {
        SurfaceSize::new(self.surface_cfg.width, self.surface_cfg.height)
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub async fn new(window: Arc<winit::window::Window>) -> Renderer<'a> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::DEBUG,
            backend_options: wgpu::BackendOptions::default(),
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let adapter_name = adapter.get_info().name.clone();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features {
                    features_wgpu: wgpu::FeaturesWGPU::POLYGON_MODE_LINE,
                    features_webgpu: wgpu::FeaturesWebGPU::empty(),
                },
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_size = window.inner_size();

        let surface_cfg = {
            let caps = surface.get_capabilities(&adapter);

            let format = caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(caps.formats[0]);

            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format,
                width: surface_size.width,
                height: surface_size.height,
                present_mode: caps.present_modes[0],
                alpha_mode: caps.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            }
        };
        surface.configure(&device, &surface_cfg);

        let depth_tex = Texture::new_depth(&device, Self::DEPTH_TEX_FORMAT, surface_size.into());

        Self {
            surface_cfg,
            surface,
            device,
            queue,
            depth_tex,
            adapter_name,
        }
    }

    pub fn update(&mut self, new_surface_size: Option<SurfaceSize>) {
        if let Some(SurfaceSize { width, height }) = new_surface_size {
            if width > 0 && height > 0 {
                self.surface_cfg.width = width;
                self.surface_cfg.height = height;
                self.surface.configure(&self.device, &self.surface_cfg);
                self.depth_tex =
                    Texture::new_depth(&self.device, Self::DEPTH_TEX_FORMAT, (width, height));
            }
        }
    }

    pub fn build_render_bundle(
        &self,
        mesh: &Mesh,
        material: &dyn ApplyMaterial,
        rt: Option<&RenderTarget>,
    ) -> wgpu::RenderBundle {
        let mut encoder = self.new_bundle_encoder(rt);
        material.apply(&mut encoder);
        mesh.draw(&mut encoder);
        encoder.finish(&wgpu::RenderBundleDescriptor { label: None })
    }

    pub fn render_pass(
        &self,
        bundles: &[wgpu::RenderBundle],
        target: Option<&RenderTarget>,
        // TODO More elegant.
        // Currently I cannot win the borrow checker and make Renderer NOT reference the Ui in some way.
        // Tried adding a lambda param here with a render pass param to allow "additional rendering"
        // but using the Ui in that lambda on the call site hits the lifetime wall.
        ui: Option<&mut Ui>,
    ) {
        let surface_tex = target.is_none().then(|| {
            self.surface
                .get_current_texture()
                // TODO Fix, this breaks on Linux when resizing.
                .expect("Missing surface texture")
        });
        let surface_tex_view = surface_tex.as_ref().map(|t| {
            t.texture
                .create_view(&wgpu::TextureViewDescriptor::default())
        });

        let color_attachment = Some(wgpu::RenderPassColorAttachment {
            view: target
                .map(|t| t.color_tex().view())
                .or(surface_tex_view.as_ref())
                .unwrap(),
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                store: wgpu::StoreOp::Store,
            },
        });

        let depth_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
            view: target
                .map(|t| t.depth_tex().view())
                .unwrap_or(self.depth_tex.view()),
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        });

        let cmd_buf = {
            let mut encoder =
                self.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[color_attachment],
                    depth_stencil_attachment: depth_attachment,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                pass.execute_bundles(bundles.iter());
                if let Some(ui) = ui {
                    ui.draw(self, &mut pass);
                }
            }

            encoder.finish()
        };

        self.queue.submit(Some(cmd_buf));

        if let Some(t) = surface_tex {
            t.present()
        }
    }

    pub fn new_uniform_bind_group(
        &self,
        data: &[u8],
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup, wgpu::Buffer) {
        let buffer = self.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let layout = self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });

        let group = self.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
        });

        (layout, group, buffer)
    }

    pub fn new_texture_bind_group(
        &self,
        texture: &Texture,
        view_dimension: wgpu::TextureViewDimension,
    ) -> (wgpu::BindGroupLayout, wgpu::BindGroup) {
        let layout = self.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: None,
        });

        let group = self.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(texture.view()),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(texture.sampler()),
                },
            ],
            label: None,
        });

        (layout, group)
    }

    fn new_bundle_encoder(&self, target: Option<&RenderTarget>) -> wgpu::RenderBundleEncoder {
        let color_format = target.map_or(self.surface_texture_format(), |t| t.color_tex().format());
        let depth_format = target.map_or(self.depth_texture_format(), |t| t.depth_tex().format());

        self.device
            .create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
                label: None,
                multiview: None,
                sample_count: 1,
                color_formats: &[Some(color_format)],
                depth_stencil: Some(wgpu::RenderBundleDepthStencil {
                    format: depth_format,
                    depth_read_only: false,
                    stencil_read_only: false,
                }),
            })
    }
}

impl Deref for Renderer<'_> {
    type Target = wgpu::Device;

    fn deref(&self) -> &Self::Target {
        &self.device
    }
}
