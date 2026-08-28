use std::iter::once;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Flex, HorizontalAlignment, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};
use tui_big_text::{BigText, PixelSize};

use crate::{
    piece::HasTile,
    powerup::{BombPowerup, PaintballPowerup, PowerUp, PowerUpType, RollerPowerup},
    screen::{
        AppScreen::{self, Lose},
        lose_screen::LoseScreen,
    },
    state::{self, BOARD_HEIGHT, BOARD_WIDTH, NEXT_LOOKUP, SCALE_Y, State},
    task::add_task,
};

#[derive(Default)]
pub struct GameScreen {}

impl GameScreen {
    pub fn init(&mut self, state: &mut State) {
        Self::add_gravity_task(state);
    }
    pub fn draw(&self, state: &mut State, frame: &mut Frame) {
        let [row] = Layout::vertical([Constraint::Length(
            (state::BOARD_HEIGHT * state::SCALE_Y + 2) as u16,
        )])
        .flex(Flex::Center)
        .areas(frame.area());

        let [left_text, left, center, right, right_text] = Layout::horizontal([
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length((state::BOARD_WIDTH * state::SCALE_X + 2) as u16),
            Constraint::Length(24),
            Constraint::Length(8),
        ])
        .spacing(2)
        .flex(Flex::Center)
        .areas(row);

        let [left_text_area] = Layout::vertical([Constraint::Length(18)])
            .flex(Flex::Center)
            .areas(left_text);
        let [right_text_area] = Layout::vertical([Constraint::Length(18)])
            .flex(Flex::Center)
            .areas(right_text);

        let lines: Vec<Line<'static>> = "CURSED"
            .chars()
            .map(|character| Line::from(character.to_string()))
            .collect();

        let left_vertical_title = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .centered()
            .style(Style::default().fg(Color::LightRed))
            .lines(lines)
            .build();
        frame.render_widget(left_vertical_title, left_text_area);

        let lines: Vec<Line<'static>> = "TETRIS"
            .chars()
            .map(|character| Line::from(character.to_string()))
            .collect();

        let right_vertical_title = BigText::builder()
            .pixel_size(PixelSize::Sextant)
            .centered()
            .style(Style::default().fg(Color::Yellow))
            .lines(lines)
            .build();
        frame.render_widget(right_vertical_title, right_text_area);

        let powerups = [
            PowerUpType::Bomb(BombPowerup::default()),
            PowerUpType::Paintball(PaintballPowerup::default()),
            PowerUpType::Roller(RollerPowerup::default()),
        ];

        let [score_area, level_area, powerup_area, _] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(powerups.len() as u16 + 3),
            Constraint::Length(12),
        ])
        .flex(Flex::Center)
        .areas(left);
        for (area, title, value) in [
            (score_area, "Score", state.score.to_string()),
            (level_area, "Level", state.level.to_string()),
        ] {
            let block = Block::bordered()
                .title(title)
                .title_alignment(HorizontalAlignment::Center);
            let content_area = block.inner(area);
            frame.render_widget(block, area);
            frame.render_widget(Paragraph::new(value).centered(), content_area);
        }

        let powerup_block = Block::bordered()
            .title("Powerup")
            .title_alignment(HorizontalAlignment::Center);
        let powerup_content = powerup_block.inner(powerup_area);
        let powerup_display = Paragraph::new(
            powerups
                .into_iter()
                .enumerate()
                .map(|(i, p_type)| {
                    Line::from(vec![
                        Span::styled(format!("[{}]", i + 1), Style::default().fg(Color::Gray)),
                        Span::from(format!(" {}", p_type.get_icon())),
                    ])
                })
                .chain(once(Line::from(vec![Span::styled(
                    format!("{} left", state.powerup.count),
                    Style::default().fg(Color::Yellow),
                )])))
                .collect::<Vec<Line>>(),
        )
        .centered();
        frame.render_widget(powerup_block, powerup_area);
        frame.render_widget(powerup_display, powerup_content);

        let game_area = state.construct_field();
        let [placed_pieces_area, next_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(SCALE_Y as u16 * 4 * NEXT_LOOKUP as u16),
        ])
        .flex(Flex::Center)
        .areas(right);

        frame.render_widget(game_area, center);
        let placed_pieces_block = Block::bordered()
            .title("Pieces / Levelup")
            .title_alignment(HorizontalAlignment::Center);
        let placed_pieces_content = placed_pieces_block.inner(placed_pieces_area);
        frame.render_widget(placed_pieces_block, placed_pieces_area);
        frame.render_widget(
            Paragraph::new(format!(
                "{} / {}",
                state.placed_pieces, state.levelup_pieces
            ))
            .centered(),
            placed_pieces_content,
        );

        let next_block = Block::bordered()
            .title("Next")
            .title_alignment(HorizontalAlignment::Center);
        let piece_spaces: [Rect; NEXT_LOOKUP] =
            Layout::vertical([Constraint::Length(6)].repeat(NEXT_LOOKUP))
                .spacing(SCALE_Y as i16)
                .areas(next_block.inner(next_area));
        for (i, piece) in piece_spaces.iter().enumerate().take(NEXT_LOOKUP) {
            frame.render_widget(
                state.piece_queue[state.piece_queue_ind + i + 1].as_widget(),
                *piece,
            );
        }
        frame.render_widget(next_block, next_area);
    }

    pub fn update(&mut self, state: &mut State) -> Option<AppScreen> {
        let collided = self.check_collision(state);
        if collided && state.game_ended {
            return Some(Lose(LoseScreen::default()));
        }
        None
    }

    fn check_collision(&mut self, state: &mut State) -> bool {
        let collided = self.has_collided(state);
        state.check_rows();
        if collided {
            if state.powerup.is_active() {
                PowerUp::on_collide(state);
            } else {
                state.blit_active_piece_to_tiles();
                state.next_piece();
            }
        }
        collided
    }

    fn has_collided(&mut self, state: &mut State) -> bool {
        if state.powerup.is_active() {
            if state.powerup.x < 0 || state.powerup.y <= 0 {
                return true;
            }

            let x = state.powerup.x as usize;
            let y = state.powerup.y as usize - 1;
            return x >= BOARD_WIDTH || y >= BOARD_HEIGHT || state.tiles[x][y].has_tile();
        }
        let p = state.piece();
        for [abs_x, abs_y] in p.abs_pos() {
            if abs_y > state::BOARD_HEIGHT as i8 {
                continue;
            }
            if abs_x < 0 || abs_x >= state::BOARD_WIDTH as i8 {
                continue;
            }
            if abs_y <= 0 || state.tiles[abs_x as usize][abs_y as usize - 1].has_tile() {
                return true;
            }
        }
        false
    }

    fn add_gravity_task(state: &mut State) {
        add_task(
            state.gravity_dur,
            |state| {
                if state.powerup.is_active() {
                    state.powerup.nudge(0, -1);
                } else {
                    state.piece().nudge(0, -1);
                }

                Self::add_gravity_task(state);
            },
            state,
        );
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
            KeyCode::Char('1') => state.activate_powerup(PowerUpType::Bomb(BombPowerup::default())),
            KeyCode::Char('2') => {
                state.activate_powerup(PowerUpType::Paintball(PaintballPowerup::default()))
            }
            KeyCode::Char('3') => {
                state.activate_powerup(PowerUpType::Roller(RollerPowerup::default()))
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
        if state.powerup.is_active() {
            state.powerup.nudge(x, y);
            if !self.validate(state) {
                state.powerup.nudge(-x, -y);
            }
        } else {
            state.piece().nudge(x, y);
            if !self.validate(state) {
                state.piece().nudge(-x, -y);
            }
        }
    }

    fn rotate_if_valid(&self, state: &mut State) {
        if state.powerup.is_active() {
            return;
        }
        state.piece().rotate();
        if !self.validate(state) {
            state.piece().unrotate();
        }
    }

    fn validate(&self, state: &mut State) -> bool {
        if state.powerup.is_active() {
            if state.powerup.x < 0 || state.powerup.y < 0 {
                return false;
            };

            let x = state.powerup.x as usize;
            let y = state.powerup.y as usize;
            return x < BOARD_WIDTH && y < BOARD_HEIGHT && !state.tiles[x][y].has_tile();
        }
        for [abs_x, abs_y] in state.piece().abs_pos() {
            let x_size = abs_x as usize;
            let y_size = abs_y as usize;
            if y_size >= BOARD_HEIGHT {
                continue;
            }
            if abs_x < 0
                || abs_y < 0
                || x_size >= BOARD_WIDTH
                || state.tiles[x_size][y_size].has_tile()
            {
                return false;
            }
        }
        true
    }
}
