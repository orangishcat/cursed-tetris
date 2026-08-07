use std::time::Duration;

use derivative::Derivative;
use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

use crate::piece::Piece;

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

    #[derivative(Default(value = "Piece::random()"))]
    pub active_piece: Piece,

    #[derivative(Default(value = "Duration::from_millis(750)"))]
    pub gravity_dur: Duration,
}

impl State {
    pub fn construct_field(&self) -> Paragraph<'static> {
        let lines: Vec<Line<'static>> = (0..BOARD_HEIGHT * SCALE_Y)
            .map(|screen_y| {
                let spans: Vec<Span<'static>> = (0..BOARD_WIDTH * SCALE_X)
                    .map(|screen_x| {
                        let tile_y = BOARD_HEIGHT - 1 - screen_y / SCALE_Y;
                        let tile_x = screen_x / SCALE_X;
                        let active = self.active_piece.is_tile_active(tile_x as i8, tile_y as i8);
                        let col = if !active {
                            self.tiles[tile_x][tile_y]
                        } else {
                            self.active_piece.color()
                        };

                        Span::styled(
                            if col != Color::Reset {
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
                .title("Game")
                .title_alignment(HorizontalAlignment::Center),
        )
    }

    pub fn blit_active_piece_to_tiles(&mut self) {
        for [abs_x, abs_y] in self.active_piece.abs_pos() {
            let x_size = abs_x as usize;
            let y_size = abs_y as usize;
            if abs_x < 0 || x_size >= BOARD_WIDTH || abs_y < 0 || y_size >= BOARD_HEIGHT {
                continue;
            }
            self.tiles[x_size][y_size] = self.active_piece.color();
        }
    }

    pub fn reset_active_piece(&mut self) {
        self.active_piece = Piece::random();
    }
}
