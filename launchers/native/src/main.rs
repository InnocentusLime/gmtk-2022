#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]

use std::io::Cursor;
use bevy::{prelude::*, window::WindowId, winit::WinitWindows};
use image;
use winit::window::Icon;
use clap::{ arg, command, Command };

fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();

    let (icon_rgba, icon_width, icon_height) = {
        let icon_buf = Cursor::new(include_bytes!("../../../assets/bevy.png"));
        let rgba = image::load(icon_buf, image::ImageFormat::Png)
            .expect("Failed to open icon path")
            .into_rgba8();

        let (width, height) = rgba.dimensions();
        let icon_raw = rgba.into_raw();
        (icon_raw, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    primary.set_window_icon(Some(icon));
}

fn main() {
    info!("Starting launcher: Native");
    let matches = command!()
        .version(my_game::VERSION)
        .name(my_game::GAME_NAME)
        .propagate_version(true)
        .subcommand(
            Command::new("run")
                .about("Runs the game")
                .args(&[
                    arg!(--inspector "Run the game with inspector turned on"),
                    arg!(--logging "Run the game with logging turned on"),
                    arg!(-d --debugging "Run the game with debugging (inspector and logging) on"),
                    arg!(-l --level [LEVEL] "Run the game with given level loaded and exit once the level is completed"),
                ])
        )
        .subcommand(
            Command::new("schedule")
                .about("Print system schedule graph and exit")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => my_game::app(
            sub_matches.contains_id("logging") || sub_matches.contains_id("debugging"),
            sub_matches.contains_id("inspector") || sub_matches.contains_id("debugging"),
            sub_matches.get_one::<String>("level").map(|x| x.as_str())
        )
        .add_startup_system(set_window_icon)
        .run(),
        Some(("schedule", _)) => bevy_mod_debugdump::print_schedule(&mut my_game::app(false, false, None)),
        None => my_game::app(false, false, None).run(),
        _ => unreachable!(),
    }
}
