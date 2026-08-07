use crossterm::event::KeyEvent;
use ratatui::widgets::{Block, Paragraph};
use ratatui::{
    Frame,
    layout::{
        Constraint,
        Flex::{self},
        HorizontalAlignment, Layout,
    },
};

use crate::state::{self, State, construct_field};

#[derive(Default)]
pub struct GameScreen {}

#[derive(Default)]
pub struct TitleScreen {}

impl GameScreen {
    fn draw(&self, state: &mut State, frame: &mut Frame) {
        let [row] = Layout::vertical([Constraint::Length(
            (state::BOARD_HEIGHT * state::SCALE_Y) as u16,
        )])
        .flex(Flex::Center)
        .areas(frame.area());

        let [left, right] = Layout::horizontal([
            Constraint::Length((state::BOARD_WIDTH * state::SCALE_X) as u16),
            Constraint::Length(12),
        ])
        .spacing(2)
        .flex(Flex::Center)
        .areas(row);

        let game_area = construct_field(state);
        let score_text = state.score.to_string();

        let [score_area] = Layout::vertical([Constraint::Length(3)]).areas(right);

        let block = Block::bordered()
            .title("Score")
            .title_alignment(HorizontalAlignment::Center);
        let content_area = block.inner(score_area);

        frame.render_widget(game_area, left);
        frame.render_widget(block, score_area);
        frame.render_widget(Paragraph::new(score_text).centered(), content_area);
    }
    fn handle_keypress(&self, state: &mut State, key: &KeyEvent) {}
}

impl TitleScreen {
    fn draw(&self, state: &mut State, frame: &mut Frame) {}
    fn handle_keypress(&self, state: &mut State, key: &KeyEvent) {}
}

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
    pub fn handle_keypress(&self, state: &mut State, key: &KeyEvent) {
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
