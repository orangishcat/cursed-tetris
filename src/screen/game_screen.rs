use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent};
use derivative::Derivative;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    screen::{
        AppScreen::{self, Lose},
        lose_screen::LoseScreen,
    },
    state::{self, BOARD_HEIGHT, BOARD_WIDTH, NEXT_LOOKUP, SCALE_Y, State},
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

        let [left, center, right] = Layout::horizontal([
            Constraint::Length(5),
            Constraint::Length((state::BOARD_WIDTH * state::SCALE_X + 2) as u16),
            Constraint::Length(24),
        ])
        .spacing(2)
        .flex(Flex::Center)
        .areas(row);

        let lines: Vec<Line<'static>> = " TETRIS"
            .chars()
            .map(|character| Line::from(character.to_string()))
            .collect();

        let vertical_title = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .centered()
            .style(Style::default())
            .lines(lines)
            .build();
        frame.render_widget(vertical_title, left);

        let game_area = state.construct_field();
        let [score_area, level_area, placed_pieces_area, next_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(SCALE_Y as u16 * 4 * NEXT_LOOKUP as u16),
        ])
        .areas(right);

        frame.render_widget(game_area, center);
        for (area, title, value) in [
            (score_area, "Score", state.score.to_string()),
            (level_area, "Level", state.level.to_string()),
            (
                placed_pieces_area,
                "Pieces / Levelup",
                format!("{} / {}", state.placed_pieces, state.levelup_pieces),
            ),
        ] {
            let block = Block::bordered()
                .title(title)
                .title_alignment(HorizontalAlignment::Center);
            let content_area = block.inner(area);

            frame.render_widget(block, area);
            frame.render_widget(Paragraph::new(value).centered(), content_area);
        }

        let next_block = Block::bordered()
            .title("Next")
            .title_alignment(HorizontalAlignment::Center);
        let piece_spaces: [Rect; NEXT_LOOKUP] =
            Layout::vertical([Constraint::Length(6)].repeat(NEXT_LOOKUP))
                .spacing(SCALE_Y as i16)
                .areas(next_block.inner(next_area));
        for i in 0..NEXT_LOOKUP {
            frame.render_widget(
                state.piece_queue[state.piece_queue_ind + i + 1].as_widget(),
                piece_spaces[i],
            );
        }
        frame.render_widget(next_block, next_area);
    }

    pub fn update(&mut self, state: &mut State) -> Option<AppScreen> {
        let collided = self.check_collision(state);
        if collided && state.game_ended {
            return Some(Lose(LoseScreen::default()));
        }
        self.update_gravity(state);
        None
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
            if abs_x < 0 || abs_x >= state::BOARD_WIDTH as i8 {
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
                let mut attempts = 0;
                while !self.check_collision(state)
                    && self.validate(state)
                    && attempts < BOARD_HEIGHT
                {
                    self.move_if_valid(state, 0, -1);
                    attempts += 1;
                }
            }
            _ => {}
        }
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
            if y_size >= BOARD_HEIGHT {
                continue;
            }
            if abs_x < 0
                || abs_y < 0
                || x_size >= BOARD_WIDTH
                || state.tiles[x_size][y_size] != Color::Reset
            {
                return false;
            }
        }
        return true;
    }
}
