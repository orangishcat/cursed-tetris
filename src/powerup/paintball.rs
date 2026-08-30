use std::time::Duration;

use ratatui::style::Color;

use crate::{config::config, piece::HasTile, state::State, task::add_task};

const PAINT_RADIUS: u8 = 4;
const WAVE_DELAY: Duration = Duration::from_millis(80);

const DIR_X: [i8; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
const DIR_Y: [i8; 8] = [0, 0, 1, -1, 1, -1, -1, 1];

#[derive(Default)]
pub struct PaintballPowerup {}

impl PaintballPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let color = state.tiles[cx as usize][cy as usize];
        Self::schedule_expansion(cx, cy, 0, color, state);
    }

    fn schedule_expansion(x: i8, y: i8, depth: u8, color: Color, state: &mut State) {
        if depth == PAINT_RADIUS {
            return;
        }
        add_task(
            WAVE_DELAY,
            move |state| {
                let config = config();
                for direction in 0..DIR_X.len() {
                    let new_x = x + DIR_X[direction];
                    let new_y = y + DIR_Y[direction];
                    if new_x < 0
                        || new_x >= config.board_width as i8
                        || new_y < 0
                        || new_y >= config.board_height as i8
                        || state.tiles[new_x as usize][new_y as usize] == color
                    {
                        continue;
                    }
                    if state.tiles[new_x as usize][new_y as usize].has_tile() {
                        state.tiles[new_x as usize][new_y as usize] = color;
                    }
                    Self::schedule_expansion(new_x, new_y, depth + 1, color, state);
                }
            },
            state,
        );
    }
}
