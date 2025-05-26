use futures_lite::future;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, Event, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::frame_time::FrameTime;
use crate::input::{Input, InputAction};
use crate::render::{Renderer, SurfaceSize};
use crate::scene::Scene;
use crate::scene::SceneCfg;
use crate::state::AppState;

#[derive(Default)]
pub struct App<'a> {
    state: Option<AppState<'a>>,
    scene: Option<Scene>,
    frame_time: Option<FrameTime>,
    new_canvas_size: Option<SurfaceSize>,
}

impl App<'_> {
    fn update_and_render(&mut self, event_loop: &ActiveEventLoop) {
        // TODO Any better way?
        let mut state = self.state.take().unwrap();
        let mut scene = self.scene.take().unwrap();

        if state.input.action_activated(InputAction::Quit) {
            event_loop.exit();
        }

        let dt = self.frame_time.as_mut().unwrap().advance();

        state.renderer.resize(self.new_canvas_size);

        scene.update(dt, &state, &self.new_canvas_size);
        scene.render(&state.renderer);

        state.input.clear();
        state.window.request_redraw();

        self.state = Some(state);
        self.scene = Some(scene);
        self.new_canvas_size = None;
    }
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // This function should be re-entrant, see the docs. Exiting if already initialized.
        if self.state.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(PhysicalSize {
                            width: 1900,
                            height: 1200,
                        })
                        .with_title("Demo"),
                )
                .unwrap(),
        );
        window.request_redraw();

        let rr = future::block_on(Renderer::new(Arc::clone(&window)));

        let state = AppState {
            window,
            renderer: rr,
            input: Input::new(),
        };

        let mut scene = Scene::new(&state);
        scene.insert_from_cfg(
            &SceneCfg::from_yaml(&String::from_utf8_lossy(include_bytes!(
                "../assets/scene.yml"
            ))),
            &state,
        );

        self.scene = Some(scene);
        self.frame_time = Some(FrameTime::new());
        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self.state.is_none() || window_id != self.state.as_ref().unwrap().window.id() {
            return;
        }

        match &event {
            WindowEvent::RedrawRequested => self.update_and_render(event_loop),
            WindowEvent::Resized(size) => self.new_canvas_size = Some(*size),
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {}
        }

        self.state
            .as_mut()
            .unwrap()
            .input
            .handle_event(Event::WindowEvent { window_id, event });
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        self.state
            .as_mut()
            .unwrap()
            .input
            .handle_event(Event::DeviceEvent { device_id, event });
    }
}
