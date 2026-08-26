use std::cmp::max;

use crate::powerup::PowerUpType::Paintball;
pub use crate::powerup::paintball::PaintballPowerup;
use crate::state::BOARD_HEIGHT;
use crate::state::BOARD_WIDTH;
use derivative::Derivative;

use crate::{
    powerup::PowerUpType::{Bomb, None},
    state::State,
};

mod bomb;
mod paintball;

pub use bomb::BombPowerup;

#[derive(Default)]
pub enum PowerUpType {
    #[default]
    None,
    Bomb(BombPowerup),
    Paintball(PaintballPowerup),
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
            _ => {}
        }
        state.powerup.reset();
        state.powerup.count -= 1;
    }

    pub fn is_type_equal(&self, p_type: &PowerUpType) -> bool {
        std::mem::discriminant(&self.p_type) == std::mem::discriminant(p_type)
    }

    pub fn is_active(&self) -> bool {
        !self.is_type_equal(&None)
    }

    pub fn nudge(&mut self, x: i8, y: i8) {
        self.x += x;
        self.y += y;
    }

    pub fn toggle_type(&mut self, p_type: PowerUpType) {
        self.p_type = if self.count <= 0 || self.is_active() {
            None
        } else {
            p_type
        };
    }

    pub fn reset(&mut self) {
        self.x = BOARD_WIDTH as i8 / 2;
        self.y = BOARD_HEIGHT as i8 - 1;
        self.p_type = None;
    }

    pub fn get_icon(&self) -> &str {
        if self.is_type_equal(&Bomb(BombPowerup::default())) {
            return "💣 ";
        }
        "🎨 "
    }
}
