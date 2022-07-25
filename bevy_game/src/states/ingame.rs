use bevy::prelude::*;
use bevy_asset_loader::*;
use bevy::sprite::Mesh2dHandle;
use iyes_loopless::prelude::*;

use super::GameState;

use crate::states::main_menu::MenuAssets;
use crate::save::Save;
use crate::level_info::LevelInfo;
use crate::level::*;
use crate::player::PlayerModification;
use crate::app::{ GameplayCamera, MenuCamera };

struct LevelCompleteCountdown(Timer);

#[derive(AssetCollection)]
pub struct InGameAssets {
    #[asset(key = "level")]
    pub level: Handle<Level>,
    #[asset(path = "Dice.glb")]
    pub player_gltf: Handle<bevy::gltf::Gltf>,
    #[asset(path = "sound/beat.ogg")]
    pub complete_sound: Handle<AudioSource>, 
}

// NOTE we can do this more cleanly in the new bevy_asset_loader version
pub struct PlayerGraphicsResources {
    pub model: Mesh2dHandle,
    pub material: Handle<ColorMaterial>,
}

impl FromWorld for PlayerGraphicsResources {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<InGameAssets>();
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

        PlayerGraphicsResources { material, model }
    }
}

fn enter() {
    info!("Entered ingame state");
}

fn death_system(
    mut commands: Commands,
    mut events: EventReader<PlayerModification>,
) {
    for ev in events.iter() {
        match ev {
            PlayerModification::Kill => {
                info!("You are dead");
                commands.insert_resource(NextState(GameState::MainMenu));
            },
            _ => (),
        }
    }
}

fn beat_system(
    mut commands: Commands,
    mut events: EventReader<PlayerModification>,
    mut save: ResMut<Save>,
    assets: Res<InGameAssets>,
    audio: Res<Audio>,
    menu_assets: Res<MenuAssets>,
    level_infos: Res<Assets<LevelInfo>>,
) {
    for ev in events.iter() {
        match ev {
            PlayerModification::Escape => {
                info!("You win");
                let level_info = level_infos.get(&menu_assets.level_info).unwrap();
                audio.play(assets.complete_sound.clone());
         
                save.register_level_complete(&*level_info);
                // TODO retry?
                match save.save() {
                    Ok(()) => (),
                    Err(e) => error!("Error recording save: {}\nThe progress will be lost.", e),
                }
                commands.insert_resource(LevelCompleteCountdown(Timer::from_seconds(2.0f32, false)));
            },
            _ => (),
        }
    }
}

fn level_complete_system(
    mut commands: Commands,
    mut timer: Option<ResMut<LevelCompleteCountdown>>,
    time: Res<Time>,
) {
    if let Some(timer) = timer.as_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.remove_resource::<LevelCompleteCountdown>();
            commands.insert_resource(NextState(GameState::MainMenu));
        }
    }
}

fn exit(
    mut commands: Commands,
    mut cam: Query<&mut Transform, With<GameplayCamera>>,
    to_del: Query<Entity, (Without<GameplayCamera>, Without<MenuCamera>)>,
) {
    info!("Exited ingame state");
    for mut tf in cam.iter_mut() { tf.translation = Vec3::new(0.0f32, 0.0f32, 50.0f32); }

    for e in to_del.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_states(app: &mut App) {
    app
        .add_enter_system(GameState::InGame, enter)
        .add_system(level_complete_system.run_in_state(GameState::InGame))
        .add_system(beat_system.run_in_state(GameState::InGame))
        .add_system(death_system.run_in_state(GameState::InGame))
        .add_exit_system(GameState::InGame, exit);
}
