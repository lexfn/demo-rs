use crate::math::{Mat4, Quat, Rotation3, Translation3, UnitQuat, Vec3};

pub enum TransformSpace {
    Local,
    World,
}

#[derive(Copy, Clone)]
pub struct Transform {
    m: Mat4,
    scale: Vec3,
    pos: Vec3,
    rot: UnitQuat,
}

// TODO Transform hierarchies
impl Transform {
    pub fn new(pos: Vec3, scale: Vec3) -> Self {
        let m = Mat4::identity();
        let rot = UnitQuat::identity();
        let mut res = Self { m, rot, scale, pos };
        res.rebuild_matrix();
        res
    }

    pub fn from_pos(pos: Vec3) -> Self {
        Transform::new(pos, Vec3::from_element(1.0))
    }

    pub fn matrix(&self) -> Mat4 {
        self.m
    }

    pub fn view_matrix(&self) -> Mat4 {
        self.m.try_inverse().unwrap()
    }

    pub fn forward(&self) -> Vec3 {
        -self.m.column(2).xyz()
    }

    pub fn right(&self) -> Vec3 {
        self.m.column(0).xyz()
    }

    pub fn up(&self) -> Vec3 {
        self.m.column(1).xyz()
    }

    // TODO This should be taken from the matrix, otherwise if this transformed is synced from
    // a rigid body then self.pos and other fields will get outdated.
    pub fn position(&self) -> Vec3 {
        self.pos
    }

    pub fn rotation_angles(&self) -> Vec3 {
        self.rot.scaled_axis()
    }

    pub fn look_at(&mut self, target: Vec3) {
        self.rot = UnitQuat::look_at_rh(&(target - self.pos), &Vec3::y_axis());
        self.rebuild_matrix();
    }

    pub fn translate(&mut self, v: Vec3) {
        self.pos += v;
        self.rebuild_matrix();
    }

    pub fn set_position(&mut self, pos: Vec3) {
        self.pos = pos;
        self.rebuild_matrix();
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        self.scale = scale;
        self.rebuild_matrix();
    }

    pub fn set(&mut self, pos: Vec3, rotation: Quat) {
        self.rot = UnitQuat::from_quaternion(rotation);
        self.pos = pos;
        self.rebuild_matrix();
    }

    pub fn rotate(&mut self, axis: Vec3, angle: f32, space: TransformSpace) {
        let axis = axis.normalize();
        let axis = match space {
            TransformSpace::Local => axis,
            TransformSpace::World => self.m.try_inverse().unwrap().transform_vector(&axis),
        };

        self.rot = UnitQuat::from_scaled_axis(axis * angle) * self.rot;
        self.rebuild_matrix();
    }

    fn rebuild_matrix(&mut self) {
        let rot_m = Rotation3::from(self.rot).transpose();
        let tr_m = Translation3::new(self.pos.x, self.pos.y, self.pos.z);
        self.m = (tr_m * rot_m)
            .to_matrix()
            .prepend_nonuniform_scaling(&self.scale);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::new(Vec3::zeros(), Vec3::from_element(1.0))
    }
}
