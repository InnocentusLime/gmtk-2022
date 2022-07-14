use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;

const TEAM_CARD_DURATION: f32 = 2.0f32;

#[derive(AssetCollection)]
pub struct ScreenAssets {
    #[asset(path = "splash/team.png")]
    team_card: Handle<Image>,
}

#[derive(Clone, Copy, Component)]
struct LogoTag;

struct SplashScreenState {
    team_card_anim_timer: Timer,
}

fn enter(mut commands: Commands, assets: Res<ScreenAssets>) {
    info!("Entered splash screen state");

    // Create the logo
    commands.spawn()
        .insert_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::Rgba {
                    red: 1.0f32,
                    green: 1.0f32,
                    blue: 1.0f32,
                    alpha: 1.0f32,
                },
                custom_size: Some(Vec2::new(2.0f32, 2.0f32)),
                ..default()
            },
            transform: Transform::from_scale(Vec3::new(0.5f32, 0.5f32, 1.0f32)),
            texture: assets.team_card.clone(),
            ..default()
        }).insert(LogoTag);

    // Initialize the splash screen state
    commands.insert_resource(SplashScreenState {
        team_card_anim_timer: Timer::from_seconds(TEAM_CARD_DURATION, false),
    });
}

fn tick(
    mut commands: Commands,
    mut state: ResMut<SplashScreenState>,
    mut logo_query: Query<&mut Sprite, With<LogoTag>>,
    time: Res<Time>,
) {
    state.team_card_anim_timer.tick(time.delta());

    // Play team card animation
    let t = state.team_card_anim_timer.percent_left();
    logo_query.single_mut().color.set_a(1.0f32 - (1.0f32 - t).sqrt());

    // Transition when done
    if state.team_card_anim_timer.finished() {
        commands.insert_resource(NextState(GameState::MainMenu));
    }
}

fn exit(
    mut commands: Commands,
    logo_query: Query<Entity, With<LogoTag>>,
) {
    info!("Exited splash screen state");

    // Despawn the logo
    commands.entity(logo_query.single()).despawn_recursive();
}

pub fn setup_states(app: &mut App) { 
    app
        .add_enter_system(GameState::SplashScreen, enter)
        .add_system(tick.run_in_state(GameState::SplashScreen))
        .add_exit_system(GameState::SplashScreen, exit);
}
