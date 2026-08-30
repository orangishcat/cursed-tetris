use std::{ops::Add, time::Duration};

use ratatui::style::Color;

use crate::{
    config::config,
    powerup::add_gravity_task,
    state::State,
    task::{add_data_task, add_task},
};

#[derive(Default)]
pub struct RollerPowerup {}

struct Node {
    x: i8,
    y: i8,
    depth: i8,
}

const WAVE_DELAY: Duration = Duration::from_millis(40);
const DELETE_DELAY: Duration = Duration::from_millis(200);

const DIR_X: [i8; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
const DIR_Y: [i8; 8] = [0, 0, 1, -1, 1, -1, -1, 1];

impl RollerPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let col = state.tiles[cx as usize][cy as usize];
        if col == Color::Reset {
            return;
        }
        Self::reach(cx, cy, col, 1, state);
        add_task(Duration::from_millis(250), add_gravity_task, state);
    }

    fn reach(x: i8, y: i8, col: Color, depth: i8, state: &mut State) {
        state.tiles[x as usize][y as usize] = Color::DarkGray;
        state.score_deleted_blocks(y as usize, 1);
        add_data_task(
            DELETE_DELAY.add(Duration::from_millis(depth as u64 * 80)),
            (x, y),
            |state, (x, y)| state.tiles[x as usize][y as usize] = Color::Reset,
            state,
        );
        Self::schedule_expansion(x, y, col, depth + 1, state);
    }

    fn schedule_expansion(x: i8, y: i8, col: Color, depth: i8, state: &mut State) {
        add_data_task(
            WAVE_DELAY,
            Node { x, y, depth },
            move |state, node| {
                let config = config();
                for i in 0..DIR_X.len() {
                    let new_x = node.x + DIR_X[i];
                    let new_y = node.y + DIR_Y[i];
                    if new_x < 0
                        || new_x >= config.board_width as i8
                        || new_y < 0
                        || new_y >= config.board_height as i8
                        || state.tiles[new_x as usize][new_y as usize] != col
                    {
                        continue;
                    }
                    Self::reach(new_x, new_y, col, node.depth, state);
                }
            },
            state,
        );
    }
}
