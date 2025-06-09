use hecs::World;

use crate::math::Vec3;
use crate::physics::Physics;
use crate::render;
use crate::render::{Renderer, Ui};
use crate::state::AppState;

use super::assets::Assets;
use super::components::{
    Camera, Grab, Hud, Materials, Mesh, Player, PlayerFocusMarker, PostProcess, RenderOrder,
    RenderTags, RigidBody, Transform, RENDER_TAG_SCENE,
};
use super::scene_config::{ComponentCfg, MaterialCfg, MeshPrefabCfg, SceneCfg};
use super::{components, materials};

pub struct Scene {
    world: World,
    physics: Physics,
    ui: Ui,
    assets: Assets,
}

impl Scene {
    pub fn new(state: &AppState) -> Self {
        let mut assets = Assets::new();
        let mut world = World::new();
        let mut physics = Physics::new();

        Player::spawn(
            &mut world,
            &state.renderer,
            &mut physics,
            Vec3::new(7.0, 7.0, 7.0),
        );
        PostProcess::spawn(&mut world, &state.renderer, &mut assets);

        let ui = Ui::new(&state.window, &state.renderer);

        Self {
            world,
            physics,
            ui,
            assets,
        }
    }

    pub fn update(&mut self, dt: f32, state: &AppState) {
        self.physics.update(dt);

        Player::update(
            dt,
            state,
            &mut self.world,
            &mut self.physics,
            &mut self.assets,
        );
        Grab::update(&mut self.world, &state.input, &mut self.physics);
        PlayerFocusMarker::update(&mut self.world);
        PostProcess::update(&mut self.world, &state.renderer, &mut self.assets);

        self.sync_physics();

        for e in state.input.new_raw_events() {
            self.ui.handle_event(e, &state.window);
        }

        Hud::update(dt, &mut self.world, state, &mut self.ui);
    }

    pub fn render(&mut self, rr: &Renderer) {
        let mut cameras = self
            .world
            .query::<(&Camera, &Transform, Option<&RenderOrder>)>();

        // Sort cameras by render order
        let mut cameras = cameras.iter().collect::<Vec<_>>();
        cameras.sort_by(|&(_, (.., ro1)), &(_, (.., ro2))| {
            ro1.unwrap_or(&RenderOrder(0))
                .0
                .partial_cmp(&ro2.unwrap_or(&RenderOrder(0)).0)
                .unwrap()
        });

        for (_, (cam, cam_tr, _)) in cameras {
            let mut items = self.world.query::<(
                &Mesh,
                &Materials,
                &Transform,
                Option<&RenderOrder>,
                Option<&RenderTags>,
            )>();

            // Pick what should be rendered by the camera
            let mut items = items
                .iter()
                .filter(|(_, (.., tag))| {
                    cam.should_render(tag.unwrap_or(&RenderTags(RENDER_TAG_SCENE)).0)
                })
                .map(|(_, (mesh, mats, tr, order, _))| (mesh, mats, tr, order))
                .collect::<Vec<_>>();

            // Sort by render order
            items.sort_by(|&(.., ro1), &(.., ro2)| {
                ro1.unwrap_or(&RenderOrder(0))
                    .0
                    .partial_cmp(&ro2.unwrap_or(&RenderOrder(0)).0)
                    .unwrap()
            });

            let bundles = items
                .into_iter()
                .map(|(mesh, mats, tr, _)| {
                    let mats = mats
                        .0
                        .iter()
                        .flatten()
                        .map(|&mat| {
                            let mat = self.assets.material(mat);
                            mat.update(rr, cam, cam_tr, tr);
                            mat.inner()
                        })
                        .collect::<Vec<_>>();
                    let mesh = self.assets.mesh(mesh.0);
                    rr.build_render_bundle(mesh, &mats, cam.target().as_ref())
                })
                .collect::<Vec<wgpu::RenderBundle>>();

            rr.render_pass(
                &bundles,
                cam.target().as_ref(),
                cam.target().is_none().then_some(&mut self.ui),
            );
        }
    }

    // TODO Try to make all initialization to happen via this func.
    pub fn insert_from_cfg(&mut self, cfg: &SceneCfg, state: &AppState) {
        for node in cfg.nodes.values() {
            let pos = node
                .pos
                .map(|pos| Vec3::from_row_slice(&pos))
                .unwrap_or(Vec3::zeros());
            let scale = node
                .scale
                .map(|scale| Vec3::from_row_slice(&scale))
                .unwrap_or(Vec3::from_element(1.0));
            let e = self.world.spawn((Transform::new(pos, scale),));
            if let Some(ro) = node.render_order {
                self.world.insert(e, (RenderOrder(ro),)).unwrap();
            }
            if let Some(rt) = node.render_tags {
                self.world.insert(e, (RenderTags(rt),)).unwrap();
            }

            if let Some(body) = &node.body {
                let body = RigidBody::cuboid(
                    components::RigidBodyParams {
                        pos,
                        rotation: Vec3::zeros(),
                        scale,
                        movable: body.movable.unwrap_or(true),
                    },
                    &mut self.physics,
                );
                self.world.insert(e, (body,)).unwrap();
            }

            if let Some(mesh) = &node.mesh {
                let mesh = if let Some(path) = &mesh.path {
                    self.assets.add_mesh_from_file(&state.renderer, path)
                } else if let Some(prefab) = &mesh.prefab {
                    match prefab {
                        MeshPrefabCfg::Quad => self
                            .assets
                            .add_mesh(render::Mesh::new_quad(&state.renderer), "quad"),
                        MeshPrefabCfg::Basis => self
                            .assets
                            .add_mesh(render::Mesh::new_basis(&state.renderer), "basis"),
                    }
                } else {
                    panic!("Unable to create mesh");
                };
                self.world.insert(e, (Mesh(mesh),)).unwrap();
            }

            if let Some(mats) = &node.materials {
                // TODO Cache, don't re-create. Currently when several nodes use the same material,
                // only one of them is rendered, must be smth with how the materials work.
                let mats = mats
                    .iter()
                    .filter_map(|mat_name| {
                        cfg.materials.iter().find_map(|m| match m {
                            MaterialCfg::Color {
                                name,
                                color: [r, g, b],
                                wireframe,
                            } if name == mat_name => {
                                let mat = materials::Material::color(
                                    &state.renderer,
                                    &mut self.assets,
                                    Vec3::new(*r, *g, *b),
                                    wireframe.unwrap_or(false),
                                );
                                Some(self.assets.add_material(mat))
                            }

                            MaterialCfg::Textured { name, texture } if name == mat_name => {
                                let mat = materials::Material::textured(
                                    &state.renderer,
                                    &mut self.assets,
                                    texture,
                                );
                                Some(self.assets.add_material(mat))
                            }

                            MaterialCfg::Skybox { name, texture } if name == mat_name => {
                                let mat = materials::Material::skybox(
                                    &state.renderer,
                                    &mut self.assets,
                                    texture,
                                );
                                Some(self.assets.add_material(mat))
                            }

                            _ => None,
                        })
                    })
                    .take(4) // Max supported materials at the moment.
                    .collect::<Vec<_>>();

                if !mats.is_empty() {
                    self.world
                        .insert(
                            e,
                            (Materials([
                                mats.first().copied(),
                                mats.get(1).copied(),
                                mats.get(2).copied(),
                                mats.get(3).copied(),
                            ]),),
                        )
                        .unwrap();
                } else {
                    panic!("Unable to create material");
                }
            }

            for cmp in node.components.as_ref().unwrap_or(&Vec::new()) {
                match cmp {
                    ComponentCfg::PlayerFocusMarker => {
                        self.world.insert(e, (PlayerFocusMarker,)).unwrap();
                    }
                }
            }
        }
    }

    fn sync_physics(&mut self) {
        for (_, (t, body)) in self.world.query_mut::<(&mut Transform, &RigidBody)>() {
            let body = self.physics.body(body.handle());
            t.set(*body.translation(), *body.rotation().inverse().quaternion());
        }
    }
}
