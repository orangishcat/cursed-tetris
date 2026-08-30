use crossterm::{
    event::{
        self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::supports_keyboard_enhancement,
};
use std::{
    io,
    time::{Duration, Instant},
};

use ratatui::DefaultTerminal;

use crate::{config::config, screen::AppScreen, state::State, task::update_tasks};

pub const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / 24);

#[derive(Default)]
pub struct App {
    state: State,
    screen: AppScreen,
    should_quit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let enhanced_keyboard = supports_keyboard_enhancement().unwrap_or(false);
        if enhanced_keyboard {
            execute!(
                terminal.backend_mut(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        }

        let result = (|| -> io::Result<()> {
            while !self.should_quit {
                let frame_start = Instant::now();
                update_tasks(&mut self.state);
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

                            match (key.code, key.modifiers) {
                                (KeyCode::Char('q'), _)
                                | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                    self.should_quit = true
                                }
                                _ => self.screen.handle_keypress(&mut self.state, &key),
                            }
                        }

                        if !event::poll(Duration::ZERO)? {
                            break;
                        }
                    }
                }

                self.screen.update(&mut self.state);
                self.should_quit |= self.screen.should_quit();
            }
            config().save();
            Ok(())
        })();

        if enhanced_keyboard {
            execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
        }
        result
    }
}
