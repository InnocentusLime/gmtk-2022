#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]

use bevy_game::*;
use clap::{ arg, command, Command };

fn main() {
    let matches = command!()
        .version(VERSION)
        .name(GAME_NAME)
        .propagate_version(true)
        .subcommand(
            Command::new("run")
                .about("Runs the game")
                .args(&[
                    arg!(-i --inspector "Run the game with inspector turned on"),
                    arg!(-l --logging "Run the game with logging turned on"),
                    arg!(-d --debugging "Run the game with debugging (inspector and logging) on"),
                ])
        )
        .subcommand(
            Command::new("schedule")
                .about("Print system schedule graph and exit")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => create_app(
            sub_matches.contains_id("logging") || sub_matches.contains_id("debugging"),
            sub_matches.contains_id("inspector") || sub_matches.contains_id("debugging"),
        ).run(),
        Some(("schedule", _)) => bevy_mod_debugdump::print_schedule(&mut create_app(false, false)),
        None => create_app(false, false).run(),
        _ => unreachable!(),
    }
}
