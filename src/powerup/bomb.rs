use std::time::Duration;

use ratatui::style::Color;

use crate::{
    config::config, piece::HasTile, powerup::add_gravity_task, state::State, task::add_task,
};

const BOMB_RADIUS: usize = 2;

#[derive(Default)]
pub struct BombPowerup {}

impl BombPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let config = config();
        let center_x = cx as usize;
        let center_y = cy as usize;
        let mut reset_tiles = vec![];
        for x in 0..config.board_width as usize {
            for y in 0..config.board_height as usize {
                if x.abs_diff(center_x) + y.abs_diff(center_y) <= BOMB_RADIUS {
                    if state.tiles[x][y].has_tile() {
                        state.score += 1;
                    }
                    state.tiles[x][y] = Color::DarkGray;
                    reset_tiles.push((x, y));
                }
            }
        }
        add_task(
            Duration::from_millis(500),
            |state| {
                for (x, y) in reset_tiles {
                    state.tiles[x][y] = Color::Reset;
                }
                add_gravity_task(state);
            },
            state,
        );
    }
}
