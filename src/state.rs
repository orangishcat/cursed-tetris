use ratatui::{
    layout::HorizontalAlignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

pub const SOLID_STR: &str = "█";
pub const LIGHT_STR: &str = "░";
pub const BLANK_STR: &str = " ";
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const SCALE_X: usize = 4;
pub const SCALE_Y: usize = 2;

#[derive(Default)]
pub struct State {
    pub score: u32,
    pub tiles: [[Color; BOARD_HEIGHT]; BOARD_WIDTH],
}

pub fn construct_field(state: &State) -> Paragraph<'static> {
    let lines: Vec<Line<'static>> = (0..BOARD_HEIGHT * SCALE_Y)
        .map(|screen_y| {
            let spans: Vec<Span<'static>> = (0..BOARD_WIDTH * SCALE_X)
                .map(|screen_x| {
                    let tile_y = screen_y / SCALE_Y;
                    let tile_x = screen_x / SCALE_X;
                    let col = state.tiles[tile_x][tile_y];

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
