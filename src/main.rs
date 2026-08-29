mod app;
mod config;
mod piece;
mod powerup;
mod screen;
mod state;
mod task;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| app::App::default().run(terminal))
}
