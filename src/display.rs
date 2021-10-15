pub const NUM_ROWS: usize = 64;
pub const NUM_COLS: usize = 32;

pub const WINDOW_WIDTH: usize = 1280;
pub const WINDOW_HEIGHT: usize = 640;

pub const CELL_WIDTH: usize = WINDOW_WIDTH / NUM_ROWS;
pub const CELL_HEIGHT: usize = WINDOW_HEIGHT / NUM_COLS;

pub type Grid = [[u8; NUM_COLS]; NUM_ROWS];

pub const fn empty_grid() -> Grid {
    [[0; NUM_COLS]; NUM_ROWS]
}
