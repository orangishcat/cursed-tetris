use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::state::State;

#[derive(Default)]
pub struct TitleScreen {}

impl TitleScreen {
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {}
    pub fn handle_keypress(&self, state: &mut State, key: &KeyEvent) {}
    pub fn update(&self, state: &mut State) {}
}
