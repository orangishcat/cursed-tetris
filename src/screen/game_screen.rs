use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use derivative::Derivative;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout},
    style::Color,
    widgets::{Block, Paragraph},
};

use crate::{
    piece::Piece,
    state::{self, BOARD_HEIGHT, BOARD_WIDTH, State},
};

#[derive(Derivative)]
#[derivative(Default)]
pub struct GameScreen {
    #[derivative(Default(value = "Instant::now()"))]
    pub last_gravity_update: Instant,
}

impl GameScreen {
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        let [row] = Layout::vertical([Constraint::Length(
            (state::BOARD_HEIGHT * state::SCALE_Y + 2) as u16,
        )])
        .flex(Flex::Center)
        .areas(frame.area());

        let [left, right] = Layout::horizontal([
            Constraint::Length((state::BOARD_WIDTH * state::SCALE_X + 2) as u16),
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
        self.check_collision(state);
        self.update_gravity(state);
    }

    fn check_collision(&mut self, state: &mut State) -> bool {
        let collided = self.has_collided(state);
        if collided {
            state.blit_active_piece_to_tiles();
            state.check_rows();
            state.next_piece();
        }
        return collided;
    }

    fn has_collided(&mut self, state: &mut State) -> bool {
        let p = state.piece();
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

        state.piece().nudge(0, -1);
        self.last_gravity_update = Instant::now();
    }

    pub fn handle_keypress(&mut self, state: &mut State, key: &KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('i') => self.rotate_if_valid(state),
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('j') => {
                self.move_if_valid(state, -1, 0)
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => {
                self.move_if_valid(state, 1, 0)
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('k') => {
                self.move_if_valid(state, 0, -1);
            }
            KeyCode::Char(' ') => {
                while !self.check_collision(state) {
                    self.move_if_valid(state, 0, -1);
                }
            }
            _ => {}
        }
    }

    fn has_lost(&self, state: &State) -> bool {
        for x in 0..BOARD_WIDTH {
            if state.tiles[x][BOARD_HEIGHT - 1] != Color::Reset {
                return true;
            }
        }
        return false;
    }

    fn move_if_valid(&self, state: &mut State, x: i8, y: i8) {
        state.piece().nudge(x, y);
        if !self.validate(state) {
            state.piece().nudge(-x, -y);
        }
    }

    fn rotate_if_valid(&self, state: &mut State) {
        state.piece().rotate();
        if !self.validate(state) {
            state.piece().unrotate();
        }
    }

    fn validate(&self, state: &mut State) -> bool {
        for [abs_x, abs_y] in state.piece().abs_pos() {
            let x_size = abs_x as usize;
            let y_size = abs_y as usize;
            if abs_x < 0
                || abs_y < 0
                || x_size >= BOARD_WIDTH
                || y_size >= BOARD_HEIGHT
                || state.tiles[x_size][y_size] != Color::Reset
            {
                return false;
            }
        }
        return true;
    }
}
