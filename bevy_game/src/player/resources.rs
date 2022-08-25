use bevy_asset_loader::asset_collection::*;
use bevy::sprite::Mesh2dHandle;
use bevy::prelude::*;

#[derive(AssetCollection)]
pub struct BasePlayerAssets {
    #[asset(path = "models/Dice.glb")]
    pub player_gltf: Handle<bevy::gltf::Gltf>,
    #[asset(path = "sound/beat.ogg")]
    pub complete_sound: Handle<AudioSource>,
}

pub struct GeneratedPlayerAssets {
    pub model: Mesh2dHandle,
    pub material: Handle<ColorMaterial>,
}

impl FromWorld for GeneratedPlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<BasePlayerAssets>();
        let gltfs = world.resource::<Assets<bevy::gltf::Gltf>>();
        let gltf_meshes = world.resource::<Assets<bevy::gltf::GltfMesh>>();
        let std_materials = world.resource::<Assets<StandardMaterial>>();

        let player_gltf = gltfs.get(&assets.player_gltf).unwrap();
        let player_gltf = gltf_meshes.get(&player_gltf.meshes[0]).unwrap();
        let player_gltf = &player_gltf.primitives[0];
        let model = Mesh2dHandle(player_gltf.mesh.clone());

        let material = ColorMaterial {
            color: Color::WHITE,
            texture: std_materials.get(&player_gltf.material.as_ref().unwrap().to_owned())
                .unwrap().base_color_texture.to_owned(),
        };
        let material = world.resource_mut::<Assets<ColorMaterial>>().add(material);

        GeneratedPlayerAssets { material, model }
    }
}
