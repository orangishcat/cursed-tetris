use std::{
    collections::{BinaryHeap, VecDeque},
    time::Duration,
};

use ratatui::style::Color;

use crate::{
    piece::HasTile,
    state::{BOARD_HEIGHT, BOARD_WIDTH, State},
    task::add_task,
};

#[derive(Default)]
pub struct RollerPowerup {}

struct Node {
    x: i8,
    y: i8,
    depth: i8,
}

const DIR_X: [i8; 4] = [1, -1, 0, 0];
const DIR_Y: [i8; 4] = [0, 0, 1, -1];

impl RollerPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let col = state.tiles[cx as usize][cy as usize];
        if col == Color::Reset {
            return;
        }
        let mut q = VecDeque::from([Node {
            x: cx,
            y: cy,
            depth: 1,
        }]);
        while let Some(node) = q.pop_front() {
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
                q.push_back(Node {
                    x: new_x,
                    y: new_y,
                    depth: node.depth + 1,
                });
                add_task(
                    Duration::from_millis(100 * node.depth as u64),
                    move |state| state.tiles[new_x as usize][new_y as usize] = Color::Reset,
                    state,
                );
            }
        }
    }
}
