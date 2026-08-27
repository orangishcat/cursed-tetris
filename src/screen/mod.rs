mod game_screen;
mod lose_screen;
mod title_screen;
use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{
    screen::{self, game_screen::GameScreen, lose_screen::LoseScreen, title_screen::TitleScreen},
    state::State,
};

pub enum AppScreen {
    Game(GameScreen),
    Title(TitleScreen),
    Lose(LoseScreen),
    Quit,
}

impl AppScreen {
    pub fn init(&mut self, state: &mut State) {
        match self {
            Self::Game(screen) => screen.init(state),
            _ => {}
        }
    }
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        match self {
            Self::Game(screen) => screen.draw(state, frame),
            Self::Title(screen) => screen.draw(state, frame),
            Self::Lose(screen) => screen.draw(state, frame),
            Self::Quit => {}
        }
    }
    pub fn update(&mut self, state: &mut State) {
        let next_screen = match self {
            Self::Game(screen) => screen.update(state),
            Self::Title(screen) => screen.update(state),
            Self::Lose(screen) => screen.update(state),
            Self::Quit => None,
        };

        if let Some(mut next_screen_inst) = next_screen {
            next_screen_inst.init(state);
            *self = next_screen_inst;
        }
    }
    pub fn handle_keypress(&mut self, state: &mut State, key: &KeyEvent) {
        match self {
            Self::Game(screen) => screen.handle_keypress(state, key),
            Self::Title(screen) => screen.handle_keypress(state, key),
            Self::Lose(screen) => screen.handle_keypress(state, key),
            Self::Quit => {}
        }
    }

    pub fn should_quit(&self) -> bool {
        matches!(self, Self::Quit)
    }
}

impl Default for AppScreen {
    fn default() -> Self {
        Self::Title(TitleScreen::default())
    }
}
