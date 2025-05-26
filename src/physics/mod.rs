mod physics;

pub type ColliderBuilder = rapier3d::prelude::ColliderBuilder;
pub type ColliderHandle = rapier3d::prelude::ColliderHandle;
pub type RigidBodyHandle = rapier3d::prelude::RigidBodyHandle;

pub use physics::{Physics, RayCastResult};
