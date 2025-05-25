use hecs::{Entity, World};
use rapier3d::dynamics::{RigidBodyBuilder, RigidBodyType};
use rapier3d::prelude::*;

use crate::input::InputAction;
use crate::math::Vec3;
use crate::physics::Physics;
use crate::render;
use crate::render::{Renderer, SurfaceSize, Ui};
use crate::scene::scene_config::{ComponentCfg, MaterialCfg, MeshPrefabCfg, SceneCfg};
use crate::state::AppState;

use super::assets::Assets;
use super::components::{
    Camera, Grab, Hud, Material, Mesh, Player, PlayerFocusMarker, RenderOrder, RenderTags,
    Transform, RENDER_TAG_SCENE,
};
use super::{components, materials, MeshHandle};

pub struct Scene {
    world: World,
    physics: Physics,
    postprocessor: Entity,
    player: Entity,
    hud: Entity,
    ui: Ui,
    box_mesh: MeshHandle,
    assets: Assets,
}

impl Scene {
    const MESH_KEY_BASIS: &'static str = "basis";
    const MESH_KEY_QUAD: &'static str = "quad";

    pub fn new(state: &AppState) -> Self {
        let mut assets = Assets::new();
        let mut world = World::new();
        let mut physics = Physics::new();

        // Player
        let player = Player::spawn(
            &mut world,
            &state.renderer,
            &mut physics,
            Vec3::new(7.0, 7.0, 7.0),
        );

        let quad_mesh = assets.add_mesh(
            render::Mesh::new_quad(&state.renderer),
            Some(Self::MESH_KEY_QUAD),
        );

        // Post-processor
        let pp_src_tex = world
            .query_one_mut::<&Camera>(player)
            .unwrap()
            .target()
            .as_ref()
            .unwrap()
            .color_tex();
        let material = materials::Material::post_process(&state.renderer, &mut assets, pp_src_tex);
        let postprocessor = world.spawn((
            Transform::default(),
            Camera::new(1.0, components::RENDER_TAG_POST_PROCESS, None),
            Mesh(quad_mesh),
            Material(assets.add_material(material)),
            RenderOrder(100),
            RenderTags(components::RENDER_TAG_POST_PROCESS),
        ));

        let hud = world.spawn((Hud,));
        let box_mesh = assets.add_mesh_from_file(&state.renderer, "cube.obj");
        let ui = Ui::new(&state.window, &state.renderer);

        Self {
            world,
            physics,
            player,
            postprocessor,
            hud,
            box_mesh,
            ui,
            assets,
        }
    }

    pub fn update(&mut self, dt: f32, state: &AppState, new_canvas_size: &Option<SurfaceSize>) {
        self.physics.update(dt);

        Player::update(dt, &mut self.world, &mut self.physics, state);
        Grab::update(&mut self.world, &state.input, &mut self.physics);
        PlayerFocusMarker::update(&mut self.world);

        if state.input.action_activated(InputAction::Spawn) {
            let player_tr = self.world.query_one_mut::<&Transform>(self.player).unwrap();
            let pos = player_tr.position() + player_tr.forward().xyz() * 5.0;
            self.spawn_box(pos, Vec3::from_element(1.0), &state.renderer);
        }

        self.sync_physics();

        if let Some(new_size) = new_canvas_size {
            self.resize(new_size, state);
        }

        for e in state.input.new_raw_events() {
            self.ui.handle_event(e, &state.window);
        }
        self.world
            .query_one_mut::<&mut Hud>(self.hud)
            .unwrap()
            .build(dt, state, &mut self.ui);
    }

    pub fn render(&mut self, rr: &Renderer) {
        self.render_with_camera(self.player, rr);
        self.render_with_camera(self.postprocessor, rr);
    }

    // TODO Continue adding other stuff until all scene initialization is done via the file.
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
            let e = self.world.spawn((
                Transform::new(pos, scale),
                RenderOrder(node.render_order),
                RenderTags(node.render_tags),
            ));

            if let Some(body_def) = &node.body {
                let movable = body_def.movable.unwrap_or(true);
                let body_type = if movable {
                    RigidBodyType::Dynamic
                } else {
                    RigidBodyType::Fixed
                };
                // TODO Move this logic into the RigidBody cmp
                let body = RigidBodyBuilder::new(body_type).translation(pos).build();
                // TODO
                let collider = ColliderBuilder::cuboid(scale.x, scale.y, scale.z)
                    .restitution(0.2)
                    .friction(0.7)
                    .build();
                let body = self.physics.add_body(body, Some(collider));
                self.world
                    .insert(
                        e,
                        (components::RigidBody {
                            handle: body,
                            movable,
                        },),
                    )
                    .unwrap();
            }

            if let Some(mesh) = &node.mesh {
                let mesh = if let Some(path) = &mesh.path {
                    self.assets.add_mesh_from_file(&state.renderer, path)
                } else if let Some(prefab) = &mesh.prefab {
                    match prefab {
                        MeshPrefabCfg::Quad => self.assets.add_mesh(
                            render::Mesh::new_quad(&state.renderer),
                            Some(Self::MESH_KEY_QUAD),
                        ),
                        MeshPrefabCfg::Basis => self.assets.add_mesh(
                            render::Mesh::new_basis(&state.renderer),
                            Some(Self::MESH_KEY_BASIS),
                        ),
                    }
                } else {
                    panic!("Unable to create mesh");
                };
                self.world.insert(e, (Mesh(mesh),)).unwrap();
            }

            if let Some(mat) = &node.material {
                // TODO Cache, don't re-create. Currently when several nodes use the same material,
                // only one of them is rendered, must be smth with how the materials work.
                let mat = cfg.materials.iter().find_map(|m| match m {
                    MaterialCfg::Color {
                        name,
                        color: [r, g, b],
                        wireframe,
                    } => {
                        if *name == mat.name {
                            let mat = materials::Material::color(
                                &state.renderer,
                                &mut self.assets,
                                Vec3::new(*r, *g, *b),
                                wireframe.unwrap_or(false),
                            );
                            Some(self.assets.add_material(mat))
                        } else {
                            None
                        }
                    }
                    MaterialCfg::Textured { name, texture } => {
                        if *name == mat.name {
                            let mat = materials::Material::textured(
                                &state.renderer,
                                &mut self.assets,
                                texture,
                            );
                            Some(self.assets.add_material(mat))
                        } else {
                            None
                        }
                    }
                    MaterialCfg::Skybox { name, texture } => {
                        if *name == mat.name {
                            let mat = materials::Material::skybox(
                                &state.renderer,
                                &mut self.assets,
                                texture,
                            );
                            Some(self.assets.add_material(mat))
                        } else {
                            None
                        }
                    }
                });

                if let Some(mat) = mat {
                    self.world.insert(e, (Material(mat),)).unwrap();
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

    // TODO Iterate over any camera, check its target and if it's configured to match the screen
    // size then resize it.
    fn resize(&mut self, new_size: &SurfaceSize, state: &AppState) {
        let mut player_cam = self.world.get::<&mut Camera>(self.player).unwrap();
        player_cam.set_aspect(new_size.width as f32 / new_size.height as f32);
        player_cam
            .target_mut()
            .unwrap()
            .resize((new_size.width, new_size.height), &state.renderer);

        let mut mat_cmp = self.world.get::<&mut Material>(self.postprocessor).unwrap();
        self.assets.remove_material(mat_cmp.0);

        let color_tex = player_cam.target().as_ref().unwrap().color_tex();
        let new_mat =
            materials::Material::post_process(&state.renderer, &mut self.assets, color_tex);
        mat_cmp.0 = self.assets.add_material(new_mat);
    }

    fn spawn_box(&mut self, pos: Vec3, scale: Vec3, rr: &Renderer) {
        let body = components::RigidBody::cuboid(
            components::RigidBodyParams {
                pos,
                scale,
                movable: true,
            },
            &mut self.physics,
        );
        let mat = materials::Material::textured(rr, &mut self.assets, "crate.png");
        self.world.spawn((
            Transform::new(pos, scale),
            Mesh(self.box_mesh),
            Material(self.assets.add_material(mat)),
            body,
            RenderOrder(0),
            RenderTags(RENDER_TAG_SCENE),
        ));
    }

    fn render_with_camera(&mut self, camera: Entity, rr: &Renderer) {
        if let Some((cam, cam_tr)) = self
            .world
            .query_one::<(&Camera, &Transform)>(camera)
            .unwrap()
            .get()
        {
            let mut items = self
                .world
                .query::<(&Mesh, &Material, &Transform, &RenderOrder, &RenderTags)>();

            // Pick what should be rendered by the camera
            let mut items = items
                .iter()
                .filter(|(_, (.., tag))| cam.should_render(tag.0))
                .map(|(_, (mesh, material, transform, order, _))| {
                    (mesh, material, transform, order)
                })
                .collect::<Vec<_>>();

            // Sort by render order
            items.sort_by(|&(.., o1), &(.., o2)| o1.0.partial_cmp(&o2.0).unwrap());

            let bundles = items
                .into_iter()
                .map(|(mesh, mat, tr, _)| {
                    let mat = self.assets.material(mat.0);
                    mat.update(rr, cam, cam_tr, tr);
                    rr.build_render_bundle(self.assets.mesh(mesh.0), mat, cam.target().as_ref())
                })
                // TODO Avoid vec allocation
                .collect::<Vec<wgpu::RenderBundle>>();

            rr.render_pass(
                &bundles,
                cam.target().as_ref(),
                cam.target().is_none().then_some(&mut self.ui),
            );
        }
    }

    fn sync_physics(&mut self) {
        for (_, (t, b)) in self
            .world
            .query_mut::<(&mut Transform, &components::RigidBody)>()
        {
            let body = self.physics.body(b.handle());
            t.set(*body.translation(), *body.rotation().inverse().quaternion());
        }
    }
}
