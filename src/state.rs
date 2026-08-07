use std::time::Duration;

use derivative::Derivative;
use rand::seq::SliceRandom;
use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::piece::{self, Piece};

pub const SOLID_STR: &str = "█";
pub const LIGHT_STR: &str = "░";
pub const BLANK_STR: &str = " ";
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const SCALE_X: usize = 4;
pub const SCALE_Y: usize = 2;

#[derive(Derivative)]
#[derivative(Default)]
pub struct State {
    pub score: u32,
    pub tiles: [[Color; BOARD_HEIGHT]; BOARD_WIDTH],
    pub piece_queue_ind: usize,

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
        self.piece().pos = [BOARD_WIDTH as i8 / 2, BOARD_HEIGHT as i8];
        self.piece_queue_ind += 1;
        if self.piece_queue_ind >= piece::QUEUE_SIZE - 3 {
            self.piece_queue_ind = 0;
            self.piece_queue.shuffle(&mut rand::rng());
        }
    }

    pub fn piece(&mut self) -> &mut Piece {
        &mut self.piece_queue[self.piece_queue_ind]
    }

    pub fn construct_field(&mut self) -> Paragraph<'static> {
        let lines: Vec<Line<'static>> = (0..BOARD_HEIGHT * SCALE_Y)
            .map(|screen_y| {
                let spans: Vec<Span<'static>> = (0..BOARD_WIDTH * SCALE_X)
                    .map(|screen_x| {
                        let tile_y = BOARD_HEIGHT - 1 - screen_y / SCALE_Y;
                        let tile_x = screen_x / SCALE_X;
                        let active = self.piece().is_tile_active(tile_x as i8, tile_y as i8);
                        let col = if !active {
                            self.tiles[tile_x][tile_y]
                        } else {
                            self.piece().color()
                        };

                        Span::styled(
                            if active {
                                LIGHT_STR
                            } else if col != Color::Reset {
                                SOLID_STR
                            } else {
                                BLANK_STR
                            },
                            Style::default().fg(col),
                        )
                    })
                    .collect();

                Line::from(spans)
            })
            .collect();

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
        let mut y = 0;
        let mut row_bonus = 10;
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
            self.score += row_bonus;
            row_bonus += 10;
            // y needs rechecking, don't increment
        }
    }
}
