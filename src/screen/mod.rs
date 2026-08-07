mod game_screen;
mod title_screen;
use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{
    screen::{self, game_screen::GameScreen, title_screen::TitleScreen},
    state::State,
};

pub enum AppScreen {
    Game(GameScreen),
    Title(TitleScreen),
}

impl AppScreen {
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        match self {
            Self::Game(screen) => screen.draw(state, frame),
            Self::Title(screen) => screen.draw(state, frame),
        }
    }
    pub fn update(&mut self, state: &mut State) {
        match self {
            Self::Game(screen) => screen.update(state),
            Self::Title(screen) => screen.update(state),
        }
    }
    pub fn handle_keypress(&mut self, state: &mut State, key: &KeyEvent) {
        match self {
            Self::Game(screen) => screen.handle_keypress(state, key),
            Self::Title(screen) => screen.handle_keypress(state, key),
        }
    }
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::Game(GameScreen::default())
    }
}
