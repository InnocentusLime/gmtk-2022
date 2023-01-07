use bevy::prelude::*;
use iyes_loopless::prelude::*;
use bevy_credits::CreditsAsset;
use bevy_asset_loader::{ asset_collection::*, dynamic_asset::* };
use crate::{LaunchParams};

use super::GameState;

#[derive(Resource, AssetCollection)]
pub struct CreditAssets {
    #[asset(path = "fonts/plain.ttf")]
    pub main_font: Handle<Font>,
    #[asset(path = "credits.cds")]
    pub credits: Handle<CreditsAsset>,
}

fn enter(
    mut commands: Commands,
    reses: Res<CreditAssets>,
    credit_assets: Res<Assets<CreditsAsset>>,
) {
    info!("Entered credits state");
    credit_assets.get(&reses.credits)
        .unwrap()
        .build(&mut commands, &reses.main_font);
}

pub fn setup_states(app: &mut App, _params: &LaunchParams) {
    app
        .add_enter_system(GameState::Credits, enter);
}
