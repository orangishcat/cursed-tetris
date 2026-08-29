use std::time::Duration;

use ratatui::style::Color;

use crate::{
    powerup::add_gravity_task,
    state::{BOARD_HEIGHT, BOARD_WIDTH, State},
    task::{add_data_task, add_task},
};

#[derive(Default)]
pub struct RollerPowerup {}

struct Node {
    x: i8,
    y: i8,
    depth: i8,
}

const DIR_X: [i8; 8] = [1, -1, 0, 0, 1, -1, 1, -1];
const DIR_Y: [i8; 8] = [0, 0, 1, -1, 1, -1, -1, 1];

impl RollerPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let col = state.tiles[cx as usize][cy as usize];
        if col == Color::Reset {
            return;
        }
        Self::recursive_delete(cx, cy, 0, col, state);
        add_task(Duration::from_millis(200), add_gravity_task, state);
    }

    pub fn recursive_delete(x: i8, y: i8, depth: i8, col: Color, state: &mut State) {
        add_data_task(
            Duration::ZERO, // this can be done without data task but whatever
            Node { x, y, depth },
            move |state, node| {
                for i in 0..DIR_X.len() {
                    let new_x = node.x + DIR_X[i];
                    let new_y = node.y + DIR_Y[i];
                    if new_x < 0
                        || new_x >= BOARD_WIDTH as i8
                        || new_y < 0
                        || new_y >= BOARD_HEIGHT as i8
                        || state.tiles[new_x as usize][new_y as usize] != col
                    {
                        continue;
                    }
                    state.tiles[new_x as usize][new_y as usize] = Color::DarkGray;
                    state.score += 1;
                    Self::recursive_delete(new_x, new_y, node.depth + 1, col, state);
                    add_task(
                        Duration::from_millis(200 + 100 * node.depth as u64),
                        move |state| state.tiles[new_x as usize][new_y as usize] = Color::Reset,
                        state,
                    );
                }
            },
            state,
        );
    }
}
