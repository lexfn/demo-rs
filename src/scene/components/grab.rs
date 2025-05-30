use hecs::World;

use crate::input::{Input, InputAction};
use crate::math::Vec3;
use crate::physics::Physics;

use super::player::Player;
use super::RigidBody;

pub struct Grab {
    // Original distance from the player when the grab was triggered.
    distance: f32,
    // Vector between the body position and the grab point at the moment of grab, both in global coordinates.
    offset: Vec3,
}

impl Grab {
    pub fn update(world: &mut World, input: &Input, physics: &mut Physics) {
        fn release_grab(world: &mut World, physics: &mut Physics) {
            let entity = if let Some((entity, (_grab, body))) =
                world.query::<(&Grab, &RigidBody)>().iter().next()
            {
                body.set_kinematic(physics, false);
                Some(entity)
            } else {
                None
            };

            if let Some(entity) = entity {
                world.remove_one::<Grab>(entity).unwrap();
            }
        }

        let (player_focus, player_focus_ray) = {
            let (_, player) = world.query_mut::<&Player>().into_iter().next().unwrap();
            (player.focus(), player.focus_ray())
        };

        if input.action_activated(InputAction::Grab) {
            if world.query::<&Grab>().iter().next().is_some() {
                release_grab(world, physics);
                return;
            }

            if let Some(player_focus) = player_focus {
                let body_entity = {
                    let mut q = world.query::<&RigidBody>();
                    let (body_entity, body) = q
                        .into_iter()
                        .find(|(_, body)| body.handle() == player_focus.body)
                        .unwrap();
                    body.set_kinematic(physics, true);
                    body_entity
                };
                let body = physics.body_mut(player_focus.body);
                let offset = *body.translation() - player_focus.point;
                world
                    .insert(
                        body_entity,
                        (Grab {
                            distance: player_focus.distance,
                            offset,
                        },),
                    )
                    .unwrap();
            }
        }

        // Update the grabbed object if any
        if let Some(player_focus_ray) = player_focus_ray {
            if let Some((_, (grab, body))) = world.query::<(&Grab, &RigidBody)>().iter().next() {
                let new_pos = player_focus_ray.point_at(grab.distance) + grab.offset;
                physics
                    .body_mut(body.handle())
                    .set_translation(new_pos.coords, true);
            }
        } else {
            // Release grab if there's no player focus anymore
            release_grab(world, physics);
        }
    }
}
