use std::sync::Arc;
use winit::window::Window;

use crate::input::Input;
use crate::render::{Renderer, SurfaceSize};

pub struct AppState<'a> {
    pub window: Arc<Window>,
    pub renderer: Renderer<'a>,
    pub input: Input,
    // TODO This should probably go into smth like "frame state"
    pub new_surface_size: Option<SurfaceSize>,
}
