use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use derivative::Derivative;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout},
    style::Color,
    widgets::{Block, Paragraph},
};

use crate::state::{self, State};

#[derive(Derivative)]
#[derivative(Default)]
pub struct GameScreen {
    #[derivative(Default(value = "Instant::now()"))]
    pub last_gravity_update: Instant,
}

impl GameScreen {
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
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

        let game_area = state.construct_field();
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

    pub fn update(&mut self, state: &mut State) {
        self.update_gravity(state);
        self.check_collision(state);
    }

    fn check_collision(&mut self, state: &mut State) {
        if self.has_collided(state) {
            state.blit_active_piece_to_tiles();
            state.reset_active_piece();
            return;
        }
    }

    fn has_collided(&mut self, state: &mut State) -> bool {
        let p = &state.active_piece;
        for [abs_x, abs_y] in p.abs_pos() {
            if abs_y - 1 >= state::BOARD_HEIGHT as i8 {
                continue;
            }
            if abs_y <= 0 || state.tiles[abs_x as usize][abs_y as usize - 1] != Color::Reset {
                return true;
            }
        }
        return false;
    }

    fn update_gravity(&mut self, state: &mut State) {
        if self.last_gravity_update.elapsed() < state.gravity_dur {
            return;
        }

        state.active_piece.nudge(0, -1);
        self.last_gravity_update = Instant::now();
    }

    pub fn handle_keypress(&self, state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('i') => state.active_piece.rotate(),
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('j') => {
                state.active_piece.nudge(-1, 0)
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                state.active_piece.nudge(1, 0)
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('k') => {
                state.active_piece.nudge(0, -1);
            }
            _ => {}
        }
    }
}
