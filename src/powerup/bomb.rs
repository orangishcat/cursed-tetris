use std::time::Duration;

use ratatui::style::Color;

use crate::{
    config::config,
    piece::HasTile,
    powerup::add_gravity_task,
    state::State,
    task::{add_data_task, add_task},
};

const BOMB_RADIUS: u8 = 2;
const WAVE_DELAY: Duration = Duration::from_millis(40);
const DELETE_DELAY: Duration = Duration::from_millis(400);

const DIR_X: [i8; 4] = [1, -1, 0, 0];
const DIR_Y: [i8; 4] = [0, 0, 1, -1];

#[derive(Default)]
pub struct BombPowerup {}

impl BombPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        Self::reach(cx, cy, 0, state);
        add_task(Duration::from_millis(50), add_gravity_task, state);
    }

    fn reach(x: i8, y: i8, depth: u8, state: &mut State) {
        if state.tiles[x as usize][y as usize].has_tile() {
            state.score_deleted_blocks(y as usize, 1);
        }
        state.tiles[x as usize][y as usize] = Color::DarkGray;
        add_data_task(
            DELETE_DELAY,
            (x, y),
            |state, (x, y)| state.tiles[x as usize][y as usize] = Color::Reset,
            state,
        );
        if depth == BOMB_RADIUS {
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
                        || state.tiles[new_x as usize][new_y as usize] == Color::DarkGray
                    {
                        continue;
                    }
                    Self::reach(new_x, new_y, depth + 1, state);
                }
            },
            state,
        );
    }
}
