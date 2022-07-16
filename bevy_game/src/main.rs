mod app;
mod level;
mod states;

use crate::app::create_app;
use crate::states::setup_states;

fn main() {
    let mut app = create_app();

    setup_states(&mut app);

    app.run();
}
