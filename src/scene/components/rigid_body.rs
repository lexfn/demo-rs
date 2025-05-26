use crate::math::Vec3;
use crate::physics::{ColliderBuilder, Physics, RigidBodyBuilder, RigidBodyHandle, RigidBodyType};

pub struct RigidBodyParams {
    pub pos: Vec3,
    pub scale: Vec3,
    pub movable: bool,
}

pub struct RigidBody {
    pub handle: RigidBodyHandle,
    pub movable: bool,
}

impl RigidBody {
    pub fn cuboid(params: RigidBodyParams, physics: &mut Physics) -> Self {
        let RigidBodyParams {
            pos,
            scale,
            movable,
        } = params;

        let body = RigidBodyBuilder::new(body_type(movable))
            .translation(Vec3::new(pos.x, pos.y, pos.z))
            .build();
        let collider = ColliderBuilder::cuboid(scale.x, scale.y, scale.z)
            .restitution(0.2)
            .friction(0.7)
            .build();

        let handle = physics.add_body(body, Some(collider));

        Self { handle, movable }
    }

    pub fn handle(&self) -> RigidBodyHandle {
        self.handle
    }

    pub fn set_kinematic(&self, physics: &mut Physics, kinematic: bool) {
        let body = physics.body_mut(self.handle);
        let new_type = if kinematic {
            RigidBodyType::KinematicPositionBased
        } else {
            body_type(self.movable)
        };
        body.set_body_type(new_type, true);
    }
}

fn body_type(movable: bool) -> RigidBodyType {
    if movable {
        RigidBodyType::Dynamic
    } else {
        RigidBodyType::Fixed
    }
}
