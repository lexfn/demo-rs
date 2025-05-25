use hecs::{With, World};

use crate::math::Vec3;

use super::{Player, RenderTags, Transform, RENDER_TAG_HIDDEN, RENDER_TAG_SCENE};

// A visual guide showing the current focus point of the player
pub struct PlayerFocusMarker;

impl PlayerFocusMarker {
    pub fn update(world: &mut World) {
        let (pos, player_pos) = {
            let (_, (player, player_tr)) = world
                .query_mut::<(&Player, &Transform)>()
                .into_iter()
                .next()
                .unwrap();
            (player.focus().map(|f| f.point), player_tr.position())
        };

        let (new_tag, new_pos, new_scale) = if let Some(pos) = pos {
            let dist_to_camera = (player_pos - pos).magnitude();
            let scale = (dist_to_camera / 10.0).clamp(0.01, 0.1);
            (RENDER_TAG_SCENE, pos, scale)
        } else {
            (RENDER_TAG_HIDDEN, Vec3::zeros(), 1.0)
        };

        let (_, (tr, tags)) = world
            .query_mut::<With<(&mut Transform, &mut RenderTags), &PlayerFocusMarker>>()
            .into_iter()
            .next()
            .unwrap();
        tr.set_position(new_pos);
        tr.set_scale(Vec3::from_element(new_scale));
        tags.0 = new_tag;
    }
}
