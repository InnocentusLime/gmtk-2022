use bevy::prelude::*;
use bevy_asset_loader::*;
use iyes_loopless::prelude::*;

use super::GameState;

fn enter() {
    info!("Entered booting state");
} 

fn exit() {
    info!("Exited booting state");
}

pub fn setup_states(app: &mut App) {
    use super::main_menu::MenuAssets;
    use super::splash_screen::ScreenAssets;
    
    app
        .add_enter_system(GameState::Booting, enter)
        .add_exit_system(GameState::Booting, exit);

    AssetLoader::new(GameState::Booting)
        .continue_to_state(GameState::SplashScreen)
        .with_collection::<ScreenAssets>()
        .with_collection::<MenuAssets>()
        .build(app);
}
