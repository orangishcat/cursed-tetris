use ratatui::style::Color;

use crate::{
    piece::HasTile,
    state::{BOARD_HEIGHT, BOARD_WIDTH, State},
};

const BOMB_RADIUS: usize = 3;

#[derive(Default)]
pub struct BombPowerup {}

impl BombPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let center_x = cx as usize;
        let center_y = cy as usize;
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if x.abs_diff(center_x) + y.abs_diff(center_y) <= BOMB_RADIUS {
                    if state.tiles[x][y].has_tile() {
                        state.score += 1;
                    }
                    state.tiles[x][y] = Color::DarkGray;
                    state.reset_queue.push([x, y]);
                }
            }
        }
    }
}
