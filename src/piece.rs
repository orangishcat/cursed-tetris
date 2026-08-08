use crate::state::{self, BLANK_STR, NEXT_LOOKUP, SCALE_X, SCALE_Y, SOLID_STR};
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

pub type Position = [i8; 2];
pub type PieceLayout = [Position; 4];

pub const PIECE_LAYOUTS: [PieceLayout; 7] = [
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

pub const PIECE_COLOR: [Color; 7] = [
    Color::Blue,
    Color::Green,
    Color::Red,
    Color::Yellow,
    Color::Magenta,
    Color::Cyan,
    Color::White,
];

pub const QUEUE_SIZE: usize = PIECE_LAYOUTS.len() * 2 + NEXT_LOOKUP;

#[derive(Default)]
pub struct Piece {
    pub id: usize,
    pub layout: PieceLayout,
    pub pos: Position,
}

impl Piece {
    pub fn from_id(id: usize) -> Self {
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
    pub fn abs_pos(&self) -> [[i8; 2]; 4] {
        self.layout.map(|[x, y]| [x + self.x(), y + self.y()])
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
    pub fn unrotate(&mut self) {
        if self.id == 3 {
            // O piece
            return;
        }
        self.layout = self.layout.map(|[x, y]| [-y, x]);
    }
    pub fn reset(&mut self) {
        *self = Piece::from_id(self.id);
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
    pub fn as_widget(&self) -> Paragraph<'static> {
        let min_x = self
            .layout
            .iter()
            .map(|position| position[0])
            .min()
            .unwrap_or(0);
        let max_x = self
            .layout
            .iter()
            .map(|position| position[0])
            .max()
            .unwrap_or(0);
        let min_y = self
            .layout
            .iter()
            .map(|position| position[1])
            .min()
            .unwrap_or(0);
        let max_y = self
            .layout
            .iter()
            .map(|position| position[1])
            .max()
            .unwrap_or(0);
        let mut lines = Vec::new();

        for y in (min_y..=max_y).rev() {
            let spans: Vec<Span<'static>> = (min_x..=max_x)
                .map(|x| {
                    let filled = self.layout.contains(&[x, y]);
                    Span::styled(
                        if filled {
                            SOLID_STR.repeat(SCALE_X)
                        } else {
                            BLANK_STR.repeat(SCALE_X)
                        },
                        Style::default().fg(self.color()),
                    )
                })
                .collect();
            let line = Line::from(spans).centered();
            for _ in 1..SCALE_Y {
                lines.push(line.clone());
            }
            lines.push(line);
        }

        Paragraph::new(lines)
    }
}
