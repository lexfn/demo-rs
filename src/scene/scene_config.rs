use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub enum ColliderShapeCfg {
    Cube,
}

#[derive(Deserialize, Debug)]
pub enum MeshPrefabCfg {
    Quad,
    Basis,
}

#[derive(Deserialize, Debug)]
pub enum ComponentCfg {
    PlayerFocusMarker,
}

#[derive(Deserialize, Debug)]
pub enum MaterialCfg {
    Color {
        name: String,
        color: [f32; 3],
        wireframe: Option<bool>,
    },
    Textured {
        name: String,
        texture: String,
    },
    Skybox {
        name: String,
        texture: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct BodyCfg {
    pub shape: ColliderShapeCfg,
    pub movable: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct MeshCfg {
    pub path: Option<String>,
    pub prefab: Option<MeshPrefabCfg>,
}

#[derive(Deserialize, Debug)]
pub struct NodeCfg {
    pub render_order: Option<i32>,
    pub render_tags: Option<u32>,
    pub pos: Option<[f32; 3]>,
    pub scale: Option<[f32; 3]>,
    pub body: Option<BodyCfg>,
    pub mesh: Option<MeshCfg>,
    pub materials: Option<Vec<String>>,
    pub components: Option<Vec<ComponentCfg>>,
}

#[derive(Deserialize, Debug)]
pub struct SceneCfg {
    pub materials: Vec<MaterialCfg>,
    pub nodes: HashMap<String, NodeCfg>,
}

impl SceneCfg {
    pub fn from_yaml(yaml: &str) -> Self {
        serde_yaml::from_str::<SceneCfg>(yaml).unwrap()
    }
}

mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let cfg = String::from_utf8_lossy(include_bytes!("../../assets/scene.yml"));
        let config = serde_yaml::from_str::<SceneCfg>(&cfg).unwrap();
        println!("{:?}", config);
    }
}
