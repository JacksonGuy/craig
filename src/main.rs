use std::io;

pub mod core;
use crate::core::app::App;

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let mut app = App::new();

    app.run(terminal)?;

    ratatui::restore();

    Ok(())
}
