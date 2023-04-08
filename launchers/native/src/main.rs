#![cfg_attr(not(debug_assertions), windows_subsystem="windows")]

use std::io::Cursor;
use bevy::{prelude::*, window::WindowId, winit::WinitWindows};
use game_lib::LaunchParams;
use winit::window::Icon;
use clap::{ Parser, Subcommand };

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

#[derive(Debug, Parser)]
#[command(name = game_lib::GAME_NAME)]
#[command(version = game_lib::VERSION)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    commands: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Run {
        #[arg(
            short,
            long = "level",
        )]
        level_file: Option<String>,
        #[arg(
            long = "logging",
            default_value_t = false,
        )]
        logging: bool,
        #[arg(
            long = "inspector",
            default_value_t = false,
        )]
        inspector: bool,
        #[arg(
            short,
            long = "debugging",
            default_value_t = false,
        )]
        debugging: bool,
    },
}

fn main() {
    info!("Starting launcher: Native");
    let args = Cli::parse();

    match args.commands {
        None => game_lib::app(default())
            .add_startup_system(set_window_icon)
            .run(),
        Some(Commands::Run {
            level_file,
            logging,
            inspector,
            debugging
        }) => {
            let params = LaunchParams {
                logging: logging || debugging,
                editor: inspector || debugging,
                level_file: level_file.as_deref(),
            };
            game_lib::app(params)
                .add_startup_system(set_window_icon)
                .run();
        },
    }
}
