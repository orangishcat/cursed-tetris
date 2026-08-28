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
}

const DIR_X: [i8; 4] = [1, -1, 0, 0];
const DIR_Y: [i8; 4] = [0, 0, 1, -1];

impl RollerPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let col = state.tiles[cx as usize][cy as usize];
        if col == Color::Reset {
            return;
        }
        Self::recursive_delete(cx, cy, col, state);
        add_task(
            Duration::from_millis(1000),
            add_gravity_task,
            state,
        );
    }

    pub fn recursive_delete(x: i8, y: i8, col: Color, state: &mut State) {
        add_data_task(
            Duration::from_millis(200),
            Node { x, y },
            move |state, data| {
                for i in 0..DIR_X.len() {
                    let new_x = data.x + DIR_X[i];
                    let new_y = data.y + DIR_Y[i];
                    if new_x < 0
                        || new_x >= BOARD_WIDTH as i8
                        || new_y < 0
                        || new_y >= BOARD_HEIGHT as i8
                        || state.tiles[new_x as usize][new_y as usize] != col
                    {
                        continue;
                    }
                    state.tiles[new_x as usize][new_y as usize] = Color::DarkGray;
                    Self::recursive_delete(new_x, new_y, col, state);
                    add_task(
                        Duration::from_millis(400),
                        move |state| state.tiles[new_x as usize][new_y as usize] = Color::Reset,
                        state,
                    );
                }
            },
            state,
        );
    }
}
