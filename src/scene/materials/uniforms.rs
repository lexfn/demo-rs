use crate::math::{Mat4, Vec3};
use crate::render;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vec3Uniform([f32; 3]);

impl Vec3Uniform {
    pub fn new(v: Vec3) -> Self {
        Self([v.x, v.y, v.z])
    }
}

impl Default for Vec3Uniform {
    fn default() -> Self {
        Self([0.0, 0.0, 1.0])
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldViewProjUniform {
    world: [[f32; 4]; 4],
    view_proj: [[f32; 4]; 4],
}

impl WorldViewProjUniform {
    pub fn new(world: &Mat4, view: &Mat4, proj: &Mat4) -> Self {
        Self {
            world: (*world).into(),
            view_proj: (render::WGPU_CONVERSION_MATRIX * proj * view).into(),
        }
    }
}

impl Default for WorldViewProjUniform {
    fn default() -> Self {
        Self {
            world: Mat4::identity().into(),
            view_proj: Mat4::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewInvProjUniform {
    // Couldn't make it work with Matrix3, probably something to do with alignment and padding
    view_mat: [[f32; 4]; 4],
    proj_mat_inv: [[f32; 4]; 4],
}

impl ViewInvProjUniform {
    pub fn new(view: &Mat4, proj: &Mat4) -> Self {
        Self {
            view_mat: (*view).into(),
            proj_mat_inv: (render::WGPU_CONVERSION_MATRIX * proj)
                .try_inverse()
                .unwrap()
                .into(),
        }
    }
}

impl Default for ViewInvProjUniform {
    fn default() -> Self {
        Self {
            view_mat: Mat4::identity().into(),
            proj_mat_inv: Mat4::identity().into(),
        }
    }
}
