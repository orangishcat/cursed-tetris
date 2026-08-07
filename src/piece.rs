use std::iter;

use crate::state::{self};
use rand::random_range;
use ratatui::style::Color;

pub type Position = [i8; 2];
pub type PieceLayout = [Position; 4];

const PIECE_LAYOUTS: [PieceLayout; 7] = [
    // I
    [[-1, 0], [0, 0], [1, 0], [2, 0]],
    // J
    [[-1, 1], [-1, 0], [0, 0], [1, 0]],
    // L
    [[1, 1], [-1, 0], [0, 0], [1, 0]],
    // O
    [[0, 1], [1, 1], [0, 0], [1, 0]],
    // S
    [[0, 1], [1, 1], [-1, 0], [0, 0]],
    // T
    [[0, 1], [-1, 0], [0, 0], [1, 0]],
    // Z
    [[-1, 1], [0, 1], [0, 0], [1, 0]],
];

const PIECE_COLOR: [Color; 7] = [
    Color::Blue,
    Color::Green,
    Color::Red,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

#[derive(Default)]
pub struct Piece {
    pub id: usize,
    pub layout: PieceLayout,
    pub pos: Position,
}

impl Piece {
    pub fn random() -> Self {
        let id = random_range(0..PIECE_LAYOUTS.len());
        Self {
            id: id,
            layout: PIECE_LAYOUTS[id],
            pos: [
                (state::BOARD_WIDTH as i8) / 2,
                state::BOARD_HEIGHT as i8 - 1,
            ],
        }
    }
    pub fn x(&self) -> i8 {
        return self.pos[0];
    }
    pub fn y(&self) -> i8 {
        return self.pos[1];
    }
    pub fn color(&self) -> Color {
        return PIECE_COLOR[self.id];
    }
    pub fn abs_pos(&self) -> impl Iterator<Item = [i8; 2]> {
        return self
            .layout
            .iter()
            .map(|[x, y]| [x + self.x(), y + self.y()]);
    }
    pub fn nudge(&mut self, x: i8, y: i8) {
        self.pos[0] += x;
        self.pos[1] += y;
    }
    pub fn rotate(&mut self) {
        if self.id == 3 {
            // O piece
            return;
        }
        self.layout = self.layout.map(|[x, y]| [y, -x]);
    }
    pub fn is_tile_active(&self, x: i8, y: i8) -> bool {
        if x.abs_diff(self.x()) > 2 || y.abs_diff(self.y()) > 2 {
            return false;
        }
        for [rel_x, rel_y] in self.layout {
            if rel_x + self.x() == x && rel_y + self.y() == y {
                return true;
            }
        }
        return false;
    }
}
