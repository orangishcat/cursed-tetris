use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::{screen::AppScreen, state::State};

#[derive(Default)]
pub struct OptionsScreen {}

impl OptionsScreen {
    pub fn draw(&self, _state: &mut State, _frame: &mut Frame) {}

    pub fn handle_keypress(&mut self, _state: &mut State, _key: &KeyEvent) {}

    pub fn update(&self, _state: &mut State) -> Option<AppScreen> {
        None
    }
}
