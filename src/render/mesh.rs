use std::io::{BufReader, Cursor};
use wgpu::util::DeviceExt;

use crate::file;

use super::vertex::{PositionUvNormalVertex, PositionUvVertex};

struct MeshPart {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl MeshPart {
    fn from_data<T: Copy + Clone + bytemuck::Pod + bytemuck::Zeroable>(
        device: &wgpu::Device,
        vertices: &[T],
        indices: &[u32],
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

pub struct Mesh {
    parts: Vec<MeshPart>,
}

impl Mesh {
    pub fn new_quad(device: &wgpu::Device) -> Self {
        Self {
            parts: vec![MeshPart::from_data(
                device,
                &[
                    // Bottom left
                    PositionUvVertex {
                        position: [-1.0, -1.0, 0.0],
                        uv: [0.0, 0.0],
                    },
                    // Top left
                    PositionUvVertex {
                        position: [-1.0, 1.0, 0.0],
                        uv: [0.0, 1.0],
                    },
                    // Top right
                    PositionUvVertex {
                        position: [1.0, 1.0, 0.0],
                        uv: [1.0, 1.0],
                    },
                    // Bottom right
                    PositionUvVertex {
                        position: [1.0, -1.0, 0.0],
                        uv: [1.0, 0.0],
                    },
                ],
                &[0, 1, 2, 0, 2, 3],
            )],
        }
    }

    // TODO Use leaner vertex format
    pub fn new_basis(device: &wgpu::Device) -> Self {
        Self {
            parts: vec![
                MeshPart::from_data(
                    device,
                    &[
                        PositionUvNormalVertex {
                            position: [0.0, 0.0, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [1.0, 0.0, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.9, 0.1, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.9, 0.0, 0.1],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                    ],
                    &[0, 1, 1, 2, 1, 3],
                ),
                MeshPart::from_data(
                    device,
                    &[
                        PositionUvNormalVertex {
                            position: [0.0, 0.0, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.0, 1.0, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.1, 0.9, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.0, 0.9, 0.1],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                    ],
                    &[0, 1, 1, 2, 1, 3],
                ),
                MeshPart::from_data(
                    device,
                    &[
                        PositionUvNormalVertex {
                            position: [0.0, 0.0, 0.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.0, 0.0, 1.0],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.1, 0.0, 0.9],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                        PositionUvNormalVertex {
                            position: [0.0, 0.1, 0.9],
                            uv: Default::default(),     // unused
                            normal: Default::default(), // unused
                        },
                    ],
                    &[0, 1, 1, 2, 1, 3],
                ),
            ],
        }
    }

    pub async fn from_data(device: &wgpu::Device, data: &str) -> Mesh {
        let cursor = futures_lite::io::Cursor::new(data);
        let mut reader = futures_lite::io::BufReader::new(cursor);

        let (meshes, _) = tobj::futures::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
            |p| async move {
                let mat_text = file::read_string_asset(p.to_str().unwrap()).await.unwrap();
                tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
            },
        )
        .await
        .unwrap();

        let parts = meshes
            .into_iter()
            .map(|m| {
                let vertices = (0..m.mesh.positions.len() / 3)
                    .map(|i| PositionUvNormalVertex {
                        position: [
                            m.mesh.positions[i * 3],
                            m.mesh.positions[i * 3 + 1],
                            m.mesh.positions[i * 3 + 2],
                        ],
                        uv: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                        normal: [
                            m.mesh.normals[i * 3],
                            m.mesh.normals[i * 3 + 1],
                            m.mesh.normals[i * 3 + 2],
                        ],
                    })
                    .collect::<Vec<_>>();

                MeshPart::from_data(device, &vertices, &m.mesh.indices)
            })
            .collect::<Vec<_>>();

        Mesh { parts }
    }

    pub fn draw<'a>(&'a self, encoder: &mut wgpu::RenderBundleEncoder<'a>) {
        for part in &self.parts {
            encoder.set_vertex_buffer(0, part.vertex_buffer.slice(..));
            encoder.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            encoder.draw_indexed(0..part.num_indices, 0, 0..1);
        }
    }
}
