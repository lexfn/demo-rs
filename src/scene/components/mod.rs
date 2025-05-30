mod camera;
mod grab;
mod hud;
mod player;
mod player_focus_marker;
mod post_process;
mod rigid_body;
mod transform;

pub use camera::Camera;
pub use grab::Grab;
pub use hud::Hud;
pub use player::Player;
pub use player_focus_marker::PlayerFocusMarker;
pub use post_process::PostProcess;
pub use rigid_body::{RigidBody, RigidBodyParams};
pub use transform::Transform;

use crate::scene::{MaterialHandle, MeshHandle};

pub struct RenderTags(pub u32);
pub struct RenderOrder(pub i32);
pub struct Mesh(pub MeshHandle);

pub struct Materials(pub [Option<MaterialHandle>; 4]);

// TODO Extract to a mod
impl Materials {
    pub fn single(handle: MaterialHandle) -> Self {
        Self([Some(handle), None, None, None])
    }
}

pub const RENDER_TAG_SCENE: u32 = 0b00000000;
pub const RENDER_TAG_POST_PROCESS: u32 = 0b00000001;
pub const RENDER_TAG_HIDDEN: u32 = 0b00000010;
