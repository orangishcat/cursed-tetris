use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{screen::AppScreen, state::State};

const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 60);

#[derive(Default)]
pub struct App {
    state: State,
    screen: AppScreen,
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let frame_start = Instant::now();
            terminal.draw(|frame| self.screen.draw(&mut self.state, frame))?;

            let timeout = FRAME_TIME.saturating_sub(frame_start.elapsed());
            if event::poll(timeout)? {
                // Drain all immediately available events.
                loop {
                    if let Event::Key(key) = event::read()? {
                        // Some platforms emit Press, Repeat, and Release events.
                        if key.kind != KeyEventKind::Press {
                            return Ok(());
                        }

                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                            _ => self.screen.handle_keypress(&mut self.state, &key),
                        }
                    }

                    if !event::poll(Duration::ZERO)? {
                        break;
                    }
                }
            }

            self.screen.update(&mut self.state);
        }
        Ok(())
    }
}
