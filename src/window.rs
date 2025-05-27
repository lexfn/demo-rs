use crate::math::Vec2;
use winit::window::CursorGrabMode;

pub trait Window {
    fn set_cursor_grabbed(&self, grabbed: bool);
    fn normalized_coordinates(&self, cursor_coords: Vec2) -> Vec2;
}

impl Window for winit::window::Window {
    fn set_cursor_grabbed(&self, grabbed: bool) {
        if grabbed {
            self.set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| self.set_cursor_grab(CursorGrabMode::Locked))
                .unwrap();
            self.set_cursor_visible(false);
        } else {
            self.set_cursor_grab(CursorGrabMode::None).unwrap();
            self.set_cursor_visible(true);
        }
    }

    fn normalized_coordinates(&self, cursor_coords: Vec2) -> Vec2 {
        let canvas_size = Vec2::new(
            self.inner_size().width as f32,
            self.inner_size().height as f32,
        );
        // Normalized device coordinates (-1..1)
        let mut ndc_coords =
            (cursor_coords.component_div(&canvas_size)) * 2.0 - Vec2::from_element(1.0);
        // Needed for some reason... Is there a bug somewhere that gets compensated by this, or is wgpu
        // NDC origin in the lower left window corner?
        ndc_coords.y *= -1.0;
        ndc_coords
    }
}
