#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]

mod app;
mod level;
mod moveable;
mod states;
mod player;
mod save;
mod tile;
mod level_info;

use crate::app::create_app;
use crate::states::setup_states;

#[cfg(feature = "schedule-print")]
fn print_schedule(app: &mut bevy::app::App) {
    bevy_mod_debugdump::print_schedule(app);    
}

fn main() {
    let mut app = create_app();
    setup_states(&mut app);

    #[cfg(feature = "schedule-print")]
    { print_schedule(&mut app); return; }
    
    app.run();
}
