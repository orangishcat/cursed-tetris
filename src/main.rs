mod app;
use std::io;

fn main() -> io::Result<()> {
    ratatui::run(|terminal| app::App::default().run(terminal))
}
