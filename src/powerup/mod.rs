use std::cmp::max;
use std::time::Duration;

use crate::piece::HasTile;
use crate::powerup::PowerUpType::Paintball;
use crate::powerup::PowerUpType::Roller;
pub use crate::powerup::paintball::PaintballPowerup;
pub use crate::powerup::roller::RollerPowerup;
use crate::state::BOARD_HEIGHT;
use crate::state::BOARD_WIDTH;
use crate::task::add_task;
use derivative::Derivative;

use crate::{
    powerup::PowerUpType::{Bomb, None},
    state::State,
};

mod bomb;
mod paintball;
mod roller;

pub use bomb::BombPowerup;

#[derive(Default)]
pub enum PowerUpType {
    #[default]
    None,
    Bomb(BombPowerup),
    Paintball(PaintballPowerup),
    Roller(RollerPowerup),
}

impl PowerUpType {
    pub fn get_icon(&self) -> &str {
        match self {
            PowerUpType::Bomb(_) => "💣 ",
            PowerUpType::Paintball(_) => "🎨 ",
            PowerUpType::Roller(_) => "🖌️ ",
            _ => " ",
        }
    }
}

#[derive(Derivative)]
#[derivative(Default)]
pub struct PowerUp {
    pub p_type: PowerUpType,

    #[derivative(Default(value = "BOARD_WIDTH as i8 / 2 "))]
    pub x: i8,
    #[derivative(Default(value = "BOARD_HEIGHT as i8 - 1"))]
    pub y: i8,

    #[derivative(Default(value = "5"))]
    pub count: u8,
}

impl PowerUp {
    pub fn on_collide(state: &mut State) {
        // tiles collide when they touch another tile, powerups collide with their center inside of a tile
        let (x, y) = (state.powerup.x, max(0, state.powerup.y - 1));
        let powerup_type = std::mem::take(&mut state.powerup.p_type);
        match powerup_type {
            Bomb(bomb) => bomb.on_collide(x, y, state),
            Paintball(paintball) => paintball.on_collide(x, y, state),
            Roller(roller) => roller.on_collide(x, y, state),
            _ => {}
        }
        state.powerup.reset();
        state.powerup.count -= 1;
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.p_type, None)
    }

    pub fn nudge(&mut self, x: i8, y: i8) {
        self.x += x;
        self.y += y;
    }

    pub fn toggle_type(&mut self, p_type: PowerUpType) {
        self.p_type = if self.count == 0 { None } else { p_type };
    }

    pub fn reset(&mut self) {
        self.x = BOARD_WIDTH as i8 / 2;
        self.y = BOARD_HEIGHT as i8 - 1;
        self.p_type = None;
    }
}

pub fn add_gravity_task(state: &mut State) {
    add_task(
        Duration::from_millis(200),
        |state| {
            let mut reschedule = false;
            for column in &mut state.tiles {
                if let Some(pos) = column.iter().position(|color| !color.has_tile())
                    && column[pos..].iter().any(|color| color.has_tile())
                {
                    column.copy_within(pos + 1..BOARD_HEIGHT, pos);
                    reschedule = true;
                }
            }
            if reschedule {
                add_gravity_task(state);
            }
        },
        state,
    );
}
