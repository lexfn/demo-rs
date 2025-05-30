use std::f32::consts::PI;

use hecs::{DynamicBundle, World};

use crate::input::{Input, InputAction};
use crate::math::{to_point3, Ray, Vec2, Vec3};
use crate::physics::{ColliderBuilder, ColliderHandle, Physics, RayCastResult, RigidBodyHandle};
use crate::render::RenderTarget;
use crate::render::Renderer;
use crate::scene::{components, materials, Assets};
use crate::state::AppState;
use crate::window::Window;

use super::camera::Camera;
use super::transform::{Transform, TransformSpace};
use super::{Materials, Mesh, RigidBody, RENDER_TAG_SCENE};

#[derive(Copy, Clone)]
pub struct PlayerFocus {
    pub point: Vec3,
    pub distance: f32,
    pub body: RigidBodyHandle,
}

pub struct Player {
    // TODO Extract into a component
    collider: ColliderHandle,
    translation_acc: Vec3,
    controlled: bool,
    // Ray and focus are separate because it's possible to have the ray but no focus (nothing is hit).
    // Ray is optional because the mouse cursor can go outside the window.
    focus_ray: Option<Ray>,
    focus: Option<PlayerFocus>,
}

impl Player {
    const SPEED: f32 = 10.0;
    const MIN_TOP_ANGLE: f32 = 0.1;
    const MIN_BOTTOM_ANGLE: f32 = PI - 0.1;
    const ROTATION_SPEED: f32 = 0.003;

    pub fn spawn(w: &mut World, rr: &Renderer, physics: &mut Physics, position: Vec3) {
        let camera = Camera::new(
            rr.surface_size().width as f32 / rr.surface_size().height as f32,
            RENDER_TAG_SCENE,
            Some(RenderTarget::new(rr, None)),
        );

        let mut tr = Transform::from_pos(position);
        tr.look_at(Vec3::from_element(0.0));

        let collider = physics.add_collider(
            ColliderBuilder::ball(0.5)
                .restitution(0.7)
                .translation(position)
                .build(),
        );

        w.spawn((
            Self {
                collider,
                translation_acc: Vec3::zeros(),
                controlled: false,
                focus_ray: None,
                focus: None,
            },
            camera,
            tr,
        ));
    }

    pub fn focus_ray(&self) -> Option<Ray> {
        self.focus_ray
    }

    pub fn focus(&self) -> Option<PlayerFocus> {
        self.focus
    }

    // TODO Introduce "scene state" or smth and pass it instead of the world/physics/assets tuple.
    pub fn update(
        dt: f32,
        state: &AppState,
        w: &mut World,
        physics: &mut Physics,
        assets: &mut Assets,
    ) {
        let (_, (tr, cam, this)) = w
            .query_mut::<(&mut Transform, &mut Camera, &mut Player)>()
            .into_iter()
            .next()
            .unwrap();

        if let Some(new_size) = state.new_surface_size {
            cam.set_aspect(new_size.width as f32 / new_size.height as f32);
            cam.target_mut()
                .unwrap()
                .resize((new_size.width, new_size.height), &state.renderer);
        }

        // Move and rotate
        if this.controlled {
            this.rotate(tr, &state.input);
            this.translate(dt, tr, &state.input, physics);
        } else {
            this.translation_acc = Vec3::zeros();
        }

        if state.input.action_activated(InputAction::ControlPlayer) {
            this.controlled = !this.controlled;
            state.window.set_cursor_grabbed(this.controlled);
        }

        this.update_focus(tr, cam, state, physics);

        if this.controlled && state.input.action_activated(InputAction::Spawn) {
            let pos = tr.position() + tr.forward().xyz() * 5.0;
            w.spawn(Self::spawn_box(pos, &state.renderer, physics, assets));
        }
    }

    fn spawn_box(
        pos: Vec3,
        rr: &Renderer,
        physics: &mut Physics,
        assets: &mut Assets,
    ) -> impl DynamicBundle {
        let scale = Vec3::from_element(1.0);
        let body = RigidBody::cuboid(
            components::RigidBodyParams {
                pos,
                scale,
                movable: true,
            },
            physics,
        );
        let mat = materials::Material::textured(rr, assets, "crate.png");
        let mesh = assets.mesh_handle("cube.obj"); // TODO Reuse some constant
        (
            Transform::new(pos, scale),
            Mesh(mesh),
            Materials([Some(assets.add_material(mat)), None, None, None]),
            body,
        )
    }

    fn translate(
        &mut self,
        dt: f32,
        transform: &mut Transform,
        input: &Input,
        physics: &mut Physics,
    ) {
        let mut translation: Vec3 = Vec3::from_element(0.0);

        if input.action_active(InputAction::MoveForward) {
            translation += transform.forward();
        }
        if input.action_active(InputAction::MoveBack) {
            translation -= transform.forward();
        }
        if input.action_active(InputAction::MoveRight) {
            translation += transform.right();
        }
        if input.action_active(InputAction::MoveLeft) {
            translation -= transform.right();
        }
        if input.action_active(InputAction::MoveUp) {
            translation += transform.up();
        }
        if input.action_active(InputAction::MoveDown) {
            translation -= transform.up();
        }

        if translation.magnitude() < 0.01 {
            return;
        }

        translation = translation.normalize() * dt * Self::SPEED;

        let (possible_translation, collider_current_pos) =
            physics.move_character(dt, translation, self.collider);
        translation = possible_translation;

        transform.translate(translation);
        physics
            .collider_mut(self.collider)
            .set_translation(collider_current_pos + translation);
    }

    fn rotate(&mut self, transform: &mut Transform, input: &Input) {
        let h_delta = input.mouse_delta().0 * Self::ROTATION_SPEED;
        let mut v_delta = input.mouse_delta().1 * Self::ROTATION_SPEED;

        // Protect from overturning: stop the camera from reaching the vertical line by small
        // margin angles.
        let angle_to_top = transform.forward().angle(&Vec3::y_axis());
        if angle_to_top + v_delta <= Self::MIN_TOP_ANGLE {
            v_delta = -(angle_to_top - Self::MIN_TOP_ANGLE);
        } else if angle_to_top + v_delta >= Self::MIN_BOTTOM_ANGLE {
            v_delta = Self::MIN_BOTTOM_ANGLE - angle_to_top;
        }

        transform.rotate(Vec3::y_axis().xyz(), h_delta, TransformSpace::World);
        transform.rotate(Vec3::x_axis().xyz(), v_delta, TransformSpace::Local);
    }

    fn update_focus(&mut self, tr: &Transform, cam: &Camera, state: &AppState, physics: &Physics) {
        let ray = if self.controlled {
            // From screen center
            Some((tr.position(), tr.forward()))
        } else if let Some(cursor_pos) = state.input.cursor_position() {
            // From cursor position
            let cursor_ndc_pos = state
                .window
                .normalized_coordinates(Vec2::new(cursor_pos.0, cursor_pos.1));
            let m = tr.matrix() * cam.proj_matrix().try_inverse().unwrap();
            let cursor_world_pos = m.transform_point(&to_point3(Vec3::new(
                cursor_ndc_pos.x,
                cursor_ndc_pos.y,
                -1.0,
            )));
            let cursor_world_pos =
                Vec3::new(cursor_world_pos.x, cursor_world_pos.y, cursor_world_pos.z);

            let orig = tr.position();
            let dir = (cursor_world_pos - orig).normalize();

            Some((tr.position(), dir))
        } else {
            None
        };

        self.focus_ray = None;
        self.focus = None;

        if let Some((orig, dir)) = ray {
            let ray = Ray::new(to_point3(orig), dir);
            self.focus_ray = Some(ray);

            if let Some(RayCastResult { distance, collider }) =
                physics.cast_ray(orig, dir, Some(self.collider))
            {
                let body = physics.collider(collider).parent().unwrap();
                self.focus = Some(PlayerFocus {
                    point: ray.point_at(distance).coords,
                    distance,
                    body,
                });
            }
        }
    }
}
