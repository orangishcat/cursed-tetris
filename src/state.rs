use std::{
    cmp::{Reverse, max},
    collections::BinaryHeap,
    time::Duration,
};

use rand::seq::SliceRandom;
use ratatui::{
    layout::HorizontalAlignment,
    style::{
        Color::{self},
        Style,
    },
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    config::config,
    piece::{self, HasTile, Piece},
    powerup::{PowerUp, PowerUpType},
    task::{Task, add_data_task},
};

pub const SOLID_STR: &str = "█";
pub const LIGHT_STR: &str = "░";
pub const BLANK_STR: &str = " ";

// todo: add border along tiles
pub const _BORDERED_STR: &str = " ▕";

pub const NEXT_LOOKUP: usize = 3;
const SCORE_EFFECT_DURATION: Duration = Duration::from_millis(700);

pub struct State {
    pub score: u32,
    pub row_score_effects: Vec<u32>,
    pub tiles: Vec<Vec<Color>>,
    pub piece_queue_ind: usize,
    pub game_ended: bool,
    pub paused: bool,
    pub placed_pieces: u32,
    pub powerup: PowerUp,

    pub task_queue: BinaryHeap<Reverse<Task>>,

    pub level: u32,

    pub levelup_pieces: u32,

    pub piece_queue: Vec<Piece>,

    pub held_piece: Option<usize>,
    pub hold_used: bool,

    pub gravity_dur: Duration,
}

impl Default for State {
    fn default() -> Self {
        let config = config();
        let level = config.start_level as u32;
        let board_width = config.board_width as usize;
        let board_height = config.board_height as usize;
        drop(config);
        Self {
            score: 0,
            row_score_effects: vec![0; board_height],
            tiles: vec![vec![Color::Reset; board_height]; board_width],
            piece_queue_ind: 0,
            game_ended: false,
            paused: false,
            placed_pieces: Self::next_levelup_count(level - 1),
            powerup: PowerUp::default(),
            task_queue: BinaryHeap::new(),
            level,
            levelup_pieces: Self::next_levelup_count(level),
            piece_queue: Self::create_pieces(),
            held_piece: None,
            hold_used: false,
            gravity_dur: Self::gravity_dur(level),
        }
    }
}

impl State {
    pub fn create_pieces() -> Vec<Piece> {
        let mut ordered_pieces: Vec<Piece> = (0..piece::QUEUE_SIZE)
            .map(|id| Piece::from_id(id % piece::PIECE_LAYOUTS.len()))
            .collect();
        ordered_pieces.shuffle(&mut rand::rng());
        ordered_pieces
    }

    pub fn next_piece(&mut self) {
        self.piece().reset();

        self.advance_piece_queue();
        self.hold_used = false;

        self.placed_pieces += 1;
        if self.placed_pieces > self.levelup_pieces {
            self.score += self.level * 10;
            self.level += 1;
            self.levelup_pieces = Self::next_levelup_count(self.level);
            self.gravity_dur = Self::gravity_dur(self.level);

            if self.level.is_multiple_of(3) {
                self.powerup.count += 1;
            }
        }
    }

    pub fn score_deleted_blocks(&mut self, row: usize, count: u32) {
        if count == 0 || row >= self.row_score_effects.len() {
            return;
        }
        self.score += count;
        self.row_score_effects[row] += count;
        add_data_task(
            SCORE_EFFECT_DURATION,
            (row, count),
            |state, (row, count)| {
                if let Some(effect) = state.row_score_effects.get_mut(row) {
                    *effect = effect.saturating_sub(count);
                }
            },
            self,
        );
    }

    fn advance_piece_queue(&mut self) {
        self.piece_queue_ind += 1;
        if self.piece_queue_ind >= piece::QUEUE_SIZE - NEXT_LOOKUP {
            self.piece_queue_ind = 0;
            self.piece_queue.rotate_right(NEXT_LOOKUP);
            self.piece_queue[NEXT_LOOKUP..].shuffle(&mut rand::rng());
        }
    }

    pub fn hold_piece(&mut self) {
        if self.hold_used || self.powerup.is_active() {
            return;
        }

        let active_id = self.piece().id;
        if let Some(held_id) = self.held_piece.replace(active_id) {
            self.piece_queue[self.piece_queue_ind] = Piece::from_id(held_id);
        } else {
            self.piece().reset();
            self.advance_piece_queue();
        }
        self.hold_used = true;
    }

    fn gravity_dur(level: u32) -> Duration {
        Duration::from_millis((750.0 * 1.0 / level as f32) as u64)
    }

    fn next_levelup_count(level: u32) -> u32 {
        if level < 1 {
            return 0;
        }
        (6_u32 * level.pow(2_u32)) + 8_u32
    }

    pub fn piece(&mut self) -> &mut Piece {
        &mut self.piece_queue[self.piece_queue_ind]
    }

    pub fn activate_powerup(&mut self, p_type: PowerUpType) {
        self.powerup.toggle_type(p_type);
        if self.powerup.count > 0 {
            self.piece().reset();
        }
    }

    pub fn construct_field(&mut self) -> Paragraph<'static> {
        let config = config();
        let (board_width, board_height) =
            (config.board_width as usize, config.board_height as usize);
        let (scale_x, scale_y) = (config.scale_x as usize, config.scale_y as usize);
        let mut lines = Vec::new();

        for y in (0..board_height).rev() {
            let spans: Vec<Span<'static>> = (0..board_width)
                .map(|x| {
                    let powerup = self.powerup.is_active()
                        && x == self.powerup.x as usize
                        && y == self.powerup.y as usize;
                    let active_piece =
                        !self.powerup.is_active() && self.piece().is_tile_active(x as i8, y as i8);
                    let col = if powerup {
                        Color::White
                    } else if active_piece {
                        self.piece().color()
                    } else {
                        self.tiles[x][y]
                    };

                    Span::styled(
                        if powerup {
                            format!(
                                "{}{}",
                                self.powerup.p_type.get_icon(),
                                BLANK_STR.repeat(max(3, scale_x) - 3)
                            )
                        } else if active_piece {
                            LIGHT_STR.repeat(scale_x)
                        } else if col != Color::Reset {
                            SOLID_STR.repeat(scale_x)
                        } else {
                            BLANK_STR.repeat(scale_x)
                        },
                        Style::default().fg(col),
                    )
                })
                .collect();
            let line = Line::from(spans);
            for _ in 1..scale_y {
                let mut clone = line.clone();
                if self.powerup.is_active() && self.powerup.y as usize == y {
                    clone.spans[self.powerup.x as usize] = Span::from(BLANK_STR.repeat(scale_x))
                }
                lines.push(clone);
            }
            lines.push(line);
        }

        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Double)
                .title("Game")
                .title_alignment(HorizontalAlignment::Center),
        )
    }

    pub fn blit_active_piece_to_tiles(&mut self) {
        let config = config();
        let (board_width, board_height) =
            (config.board_width as usize, config.board_height as usize);
        let (positions, color) = {
            let p = self.piece();
            (p.abs_pos(), p.color())
        };
        for [abs_x, abs_y] in positions {
            let x = abs_x as usize;
            let y = abs_y as usize;
            if abs_x < 0 || x >= board_width || abs_y < 0 || y >= board_height {
                continue;
            }
            self.tiles[x][y] = color;
        }
    }

    pub fn check_rows(&mut self) {
        let config = config();
        let (board_width, board_height) =
            (config.board_width as usize, config.board_height as usize);
        self.eliminate_full_rows();
        if (0..board_width).any(|x| self.tiles[x][board_height - 1].has_tile()) {
            self.game_ended = true;
        }
    }

    pub fn eliminate_full_rows(&mut self) {
        let config = config();
        let (board_width, board_height) =
            (config.board_width as usize, config.board_height as usize);
        let mut y = 0;
        while y < board_height {
            let full_row = (0..board_width).all(|x| self.tiles[x][y].has_tile());
            if !full_row {
                y += 1;
                continue;
            }

            for column in &mut self.tiles {
                column.copy_within(y + 1..board_height, y);
                column[board_height - 1] = Color::Reset;
            }
            self.score_deleted_blocks(y, board_width as u32);
            break;
        }
    }
}
