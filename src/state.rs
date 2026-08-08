use std::time::Duration;

use derivative::Derivative;
use rand::seq::SliceRandom;
use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    piece::{self, Piece},
    powerup::PowerUp,
};

pub const SOLID_STR: &str = "█";
pub const LIGHT_STR: &str = "░";
pub const BLANK_STR: &str = " ";
pub const PIECES_PER_LEVEL: i32 = 16;
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const SCALE_X: usize = 4;
pub const SCALE_Y: usize = 2;
pub const NEXT_LOOKUP: usize = 3;

#[derive(Derivative)]
#[derivative(Default)]
pub struct State {
    pub score: u32,
    pub tiles: [[Color; BOARD_HEIGHT]; BOARD_WIDTH],
    pub piece_queue_ind: usize,
    pub game_ended: bool,
    pub placed_pieces: i32,
    pub powerup: PowerUp,

    #[derivative(Default(value = "1"))]
    pub level: i32,

    #[derivative(Default(value = "PIECES_PER_LEVEL"))]
    pub levelup_pieces: i32,

    #[derivative(Default(value = "State::create_pieces()"))]
    pub piece_queue: Vec<Piece>,

    #[derivative(Default(value = "Duration::from_millis(750)"))]
    pub gravity_dur: Duration,
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

        self.piece_queue_ind += 1;
        if self.piece_queue_ind >= piece::QUEUE_SIZE - NEXT_LOOKUP {
            self.piece_queue_ind = 0;
            self.piece_queue.rotate_right(NEXT_LOOKUP);
            self.piece_queue[NEXT_LOOKUP..].shuffle(&mut rand::rng());
        }

        self.placed_pieces += 1;
        if self.placed_pieces > self.levelup_pieces {
            self.level += 1;
            self.levelup_pieces = self.next_levelup_count();
            self.gravity_dur =  // custom exponenetial curve for gravity ms
                Duration::from_millis((750.0 * (self.level as f64).powf(-0.68144)) as u64);
        }
    }

    fn next_levelup_count(&self) -> i32 {
        ((PIECES_PER_LEVEL as f32) * (self.level as f32).powf(1.25)) as i32
    }

    pub fn piece(&mut self) -> &mut Piece {
        &mut self.piece_queue[self.piece_queue_ind]
    }

    pub fn construct_field(&mut self) -> Paragraph<'static> {
        let mut lines = Vec::new();

        for y in (0..BOARD_HEIGHT).rev() {
            let spans: Vec<Span<'static>> = (0..BOARD_WIDTH)
                .map(|x| {
                    let powerup = self.powerup.is_active()
                        && x == self.powerup.x as usize
                        && y == self.powerup.y as usize;
                    let active_piece =
                        !self.powerup.is_active() && self.piece().is_tile_active(x as i8, y as i8);
                    let col = if powerup {
                        Color::White
                    } else if !active_piece {
                        self.tiles[x][y]
                    } else {
                        self.piece().color()
                    };

                    Span::styled(
                        if powerup {
                            String::from(self.powerup.get_icon())
                        } else if active_piece {
                            LIGHT_STR.repeat(SCALE_X)
                        } else if col != Color::Reset {
                            SOLID_STR.repeat(SCALE_X)
                        } else {
                            BLANK_STR.repeat(SCALE_X)
                        },
                        Style::default().fg(col),
                    )
                })
                .collect();
            let line = Line::from(spans);
            for _ in 1..SCALE_Y {
                lines.push(line.clone());
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
        let (positions, color) = {
            let p = self.piece();
            (p.abs_pos(), p.color())
        };
        for [abs_x, abs_y] in positions {
            let x = abs_x as usize;
            let y = abs_y as usize;
            if abs_x < 0 || x >= BOARD_WIDTH || abs_y < 0 || y >= BOARD_HEIGHT {
                continue;
            }
            self.tiles[x][y] = color;
        }
    }

    pub fn check_rows(&mut self) {
        self.eliminate_full_rows();
        if (0..BOARD_WIDTH).any(|x| self.tiles[x][BOARD_HEIGHT - 1] != Color::Reset) {
            self.game_ended = true;
        }
    }

    pub fn eliminate_full_rows(&mut self) {
        let mut y = 0;
        while y < BOARD_HEIGHT {
            let full_row = (0..BOARD_WIDTH).all(|x| self.tiles[x][y] != Color::Reset);
            if !full_row {
                y += 1;
                continue;
            }

            for column in &mut self.tiles {
                column.copy_within(y + 1..BOARD_HEIGHT, y);
                column[BOARD_HEIGHT - 1] = Color::Reset;
            }
            self.score += BOARD_WIDTH as u32;
            break;
        }
    }
}
