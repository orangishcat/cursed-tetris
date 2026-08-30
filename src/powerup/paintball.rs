use crate::{config::config, piece::HasTile, state::State};

const PAINT_RADIUS: usize = 4;

#[derive(Default)]
pub struct PaintballPowerup {}

impl PaintballPowerup {
    pub fn on_collide(&self, cx: i8, cy: i8, state: &mut State) {
        let config = config();
        let center_x = cx as usize;
        let center_y = cy as usize;
        let col = state.tiles[center_x][center_y];
        for x in 0..config.board_width as usize {
            for y in 0..config.board_height as usize {
                if x.abs_diff(center_x) + y.abs_diff(center_y) <= PAINT_RADIUS
                    && state.tiles[x][y].has_tile()
                {
                    state.tiles[x][y] = col;
                }
            }
        }
    }
}
