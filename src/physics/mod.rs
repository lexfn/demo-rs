mod physics;

use rapier3d::prelude;

pub type ColliderBuilder = prelude::ColliderBuilder;
pub type RigidBodyBuilder = prelude::RigidBodyBuilder;
pub type ColliderHandle = prelude::ColliderHandle;
pub type RigidBodyHandle = prelude::RigidBodyHandle;
pub type RigidBodyType = prelude::RigidBodyType;

pub use physics::{Physics, RayCastResult};
