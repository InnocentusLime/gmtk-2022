# What is this

This is a puzzle game, main mechanic of which involves flipping around a piece of dice! Depending on which number will be facing up after that flip, the level will change.

# The origin of the project

This project started out as an entry for the GMTK (Game Maker's Tool Kit) game jam of 2022. The theme of the jam was "Roll of the dice". You can find the original submission on [the jam website](https://madtfoxy.itch.io/gluttony). 

After receiving some small positive feedback, I decided to finish this game.

# Technical info

Right now the game is using [the bevy game engine](https://bevyengine.org/) version 0.8.1, along with some third party plugins. For complete list of dependencies see the [`Cargo.toml`](Cargo.toml) manifest file, located in the project root.

The levels of the game were created via [the tiled level editor](https://www.mapeditor.org/).

The game has different modes you can boot it up in. For more info simply run the game like this:

```bash
game_excutable --help
```

# The state of this project

The game is still under active development. Since it is my first game project, I can't really guarantee when it will be out, but I am planning to have some key work done by the end of October.

# Getting the game

The game is avaliable for free under the conditions of the **MIT license**. There are 2 ways for you to get a build at the moment.

## Download the latest release

1. Go to [the latest release page](https://github.com/InnocentusLime/gmtk-2022/releases/latest)
2. Pick the build, that matches your platform, according to this table (apologies for the inconvenient names, that'll be fixed)

| Windows | Linux | MAC |
| --- | --- | --- |
| my_game-v0.8.1-windows | my_game-v0.8.1-ubuntu | my_game-v0.8.1-darwin |

## Building the game yourself

1. Clone the repository and select the branch you want to build[^1]
2. Install Rust with version not older than 1.64.x. Installation guide: https://www.rust-lang.org/tools/install
3. Make sure you have `cargo` installed
4. If you are building on Linux, make sure you have `libasound2-dev` and `libudev-dev` installed (these packages are required by the bevy game engine)
5. Run `cargo build --release`
6. The result of the build is a single executable located in `target/release`. Alternatively, you can run the game simply by writing `cargo run`

[^1]: **KEEP IN MIND**, that the `main` branch might not contain a working version of the game. Be aware, that all other branches except `main` might not even compile.
