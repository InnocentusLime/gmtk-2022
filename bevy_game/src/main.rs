mod app;
mod level;
mod states;
mod player;
mod special_tiles;

use crate::app::create_app;
use crate::states::setup_states;

fn main() {
    let mut app = create_app();

    setup_states(&mut app);

    app.run();
}
