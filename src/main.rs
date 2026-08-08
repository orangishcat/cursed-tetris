mod app;
mod piece;
mod powerup;
mod screen;
mod state;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| app::App::default().run(terminal))
}
