use macroquad::prelude::*;

pub const TILE: f32 = 48.0;
pub const COLS: i32 = 20;
pub const ROWS: i32 = 12;

pub const TOP_BAR: f32 = 48.0;
pub const MAP_X: f32 = 0.0;
pub const MAP_Y: f32 = TOP_BAR;
pub const MAP_W: f32 = COLS as f32 * TILE;
pub const MAP_H: f32 = ROWS as f32 * TILE;
pub const SIDEBAR_X: f32 = MAP_W;
pub const SIDEBAR_W: f32 = 240.0;
pub const SCREEN_W: f32 = MAP_W + SIDEBAR_W;
pub const SCREEN_H: f32 = TOP_BAR + MAP_H;

/// Corners of the enemy path in grid coordinates. The first and last points
/// sit just outside the map so enemies walk in and out of view.
const PATH: [(i32, i32); 10] = [
    (-1, 2),
    (4, 2),
    (4, 8),
    (9, 8),
    (9, 3),
    (14, 3),
    (14, 9),
    (17, 9),
    (17, 5),
    (20, 5),
];

pub fn tile_center(col: i32, row: i32) -> Vec2 {
    vec2(
        MAP_X + (col as f32 + 0.5) * TILE,
        MAP_Y + (row as f32 + 0.5) * TILE,
    )
}

/// Converts a screen position to a tile on the map, if it is on one.
pub fn tile_at(pos: Vec2) -> Option<(i32, i32)> {
    let col = ((pos.x - MAP_X) / TILE).floor() as i32;
    let row = ((pos.y - MAP_Y) / TILE).floor() as i32;
    in_bounds(col, row).then_some((col, row))
}

pub fn in_bounds(col: i32, row: i32) -> bool {
    (0..COLS).contains(&col) && (0..ROWS).contains(&row)
}

pub struct Map {
    pub waypoints: Vec<Vec2>,
    path_tiles: Vec<bool>,
}

impl Map {
    pub fn new() -> Self {
        let mut path_tiles = vec![false; (COLS * ROWS) as usize];
        for pair in PATH.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            let (dc, dr) = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
            let (mut c, mut r) = a;
            loop {
                if in_bounds(c, r) {
                    path_tiles[(r * COLS + c) as usize] = true;
                }
                if (c, r) == b {
                    break;
                }
                c += dc;
                r += dr;
            }
        }
        Self {
            waypoints: PATH.iter().map(|&(c, r)| tile_center(c, r)).collect(),
            path_tiles,
        }
    }

    pub fn is_path(&self, col: i32, row: i32) -> bool {
        in_bounds(col, row) && self.path_tiles[(row * COLS + col) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_is_marked() {
        let map = Map::new();
        assert!(map.is_path(0, 2));
        assert!(map.is_path(4, 5));
        assert!(map.is_path(19, 5));
        assert!(!map.is_path(0, 0));
        assert!(!map.is_path(-1, 2));
    }

    #[test]
    fn tile_round_trip() {
        let p = tile_center(7, 4);
        assert_eq!(tile_at(p), Some((7, 4)));
        assert_eq!(tile_at(vec2(MAP_W + 5.0, 100.0)), None);
    }
}
