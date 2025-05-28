use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod file;
mod frame_time;
mod input;
mod math;
mod physics;
mod render;
mod scene;
mod state;
mod window;

// TODO Encapsulate/isolate library stuff (e.g. Rapier, wgpu, etc.) from the rest of the code better.
// TODO First-person controller.
// TODO Spawned boxes should be rotated based on the camera view.
// TODO Dragging should maintain box rotation relative to the camera.
// TODO Selected object highlighting.
// TODO Gizmos (e.g. axes instead of a box representing the player's target).
// TODO Switch to raw Vulkan and/or introduce it as a separate backend. wgpu has an unstable API.

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Error: {e}");
    }
}
