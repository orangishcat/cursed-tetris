use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::io;

use ratatui::{DefaultTerminal, Frame};

use crate::{screen::AppScreen, state::State};

#[derive(Default)]
pub struct App {
    state: State,
    screen: AppScreen,
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_event()?;
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        self.screen.draw(&mut self.state, frame);
    }

    pub fn handle_event(&mut self) -> io::Result<()> {
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
        Ok(())
    }
}
