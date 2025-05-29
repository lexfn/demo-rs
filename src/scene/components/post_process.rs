use super::{
    super::materials::Material, super::Assets, Camera, Materials, Mesh, Player,
    RenderOrder, RenderTags, Transform, RENDER_TAG_POST_PROCESS,
};
use crate::render;
use crate::render::Renderer;
use hecs::World;

pub struct PostProcess;

impl PostProcess {
    pub fn spawn(w: &mut World, rr: &Renderer, assets: &mut Assets) {
        let mat = {
            let src_tex = Self::player_camera_mut(w)
                .target()
                .as_ref()
                .unwrap()
                .color_texture();
            Material::post_process(rr, assets, src_tex)
        };
        let mesh = assets.add_mesh(render::Mesh::new_quad(rr), "quad");

        w.spawn((
            PostProcess,
            Transform::default(),
            Camera::new(1.0, RENDER_TAG_POST_PROCESS, None),
            Mesh(mesh),
            Materials::single(assets.add_material(mat)),
            RenderOrder(100),
            RenderTags(RENDER_TAG_POST_PROCESS),
        ));
    }

    pub fn update(w: &mut World, rr: &Renderer, assets: &mut Assets) {
        let new_mat = {
            let player_cam = Self::player_camera(w);
            let tex = player_cam.target().as_ref().unwrap().color_texture();
            Material::post_process(rr, assets, tex)
        };
        let (_, mats) = w
            .query_mut::<&mut Materials>()
            .with::<&Self>()
            .into_iter()
            .next()
            .unwrap();
        assets.remove_material(mats.0.get_mut(0).unwrap().unwrap());
        mats.0[0] = Some(assets.add_material(new_mat));
    }

    fn player_camera_mut(w: &mut World) -> &mut Camera {
        w.query_mut::<&mut Camera>()
            .with::<&Player>()
            .into_iter()
            .next()
            .unwrap()
            .1
    }

    fn player_camera(w: &mut World) -> &Camera {
        w.query_mut::<&Camera>()
            .with::<&Player>()
            .into_iter()
            .next()
            .unwrap()
            .1
    }
}
