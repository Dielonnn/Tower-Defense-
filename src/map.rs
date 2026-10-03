use macroquad::prelude::*;

/// Size of the old grid cell, still used as the unit for ranges.
pub const TILE: f32 = 48.0;

pub const TOP_BAR: f32 = 48.0;
pub const MAP_X: f32 = 0.0;
pub const MAP_Y: f32 = TOP_BAR;
pub const MAP_W: f32 = 960.0;
pub const MAP_H: f32 = 672.0;
pub const SIDEBAR_X: f32 = MAP_X + MAP_W;
pub const SIDEBAR_W: f32 = 280.0;
pub const SCREEN_W: f32 = MAP_W + SIDEBAR_W;
pub const SCREEN_H: f32 = TOP_BAR + MAP_H;

pub const PATH_WIDTH: f32 = 40.0;
pub const TOWER_RADIUS: f32 = 18.0;

/// The default map, traced from a 512x512 sketch. It enters from the left,
/// loops back over itself twice and leaves through the bottom.
const SKETCH: [(f32, f32); 13] = [
    (-24.0, 236.0),
    (190.0, 236.0),
    (188.0, 84.0),
    (80.0, 89.0),
    (80.0, 397.0),
    (265.0, 392.0),
    (257.0, 292.0),
    (36.0, 301.0),
    (48.0, 466.0),
    (362.0, 462.0),
    (362.0, 84.0),
    (461.0, 83.0),
    (467.0, 545.0),
];

/// The original zigzag map, in 48px grid cells.
const ZIGZAG: [(i32, i32); 10] = [
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

pub const MAP_COUNT: usize = 2;

fn from_sketch(x: f32, y: f32) -> Vec2 {
    vec2(MAP_X + x * MAP_W / 512.0, MAP_Y + y * MAP_H / 512.0)
}

fn grid_center(col: i32, row: i32) -> Vec2 {
    vec2(
        MAP_X + (col as f32 + 0.5) * TILE,
        MAP_Y + (row as f32 + 0.5) * TILE,
    )
}

pub fn in_map(pos: Vec2, margin: f32) -> bool {
    pos.x >= MAP_X + margin
        && pos.x <= MAP_X + MAP_W - margin
        && pos.y >= MAP_Y + margin
        && pos.y <= MAP_Y + MAP_H - margin
}

fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    (a + ab * t).distance(p)
}

/// Distance from `p` to the segment `a`-`b`, exposed for hit tests.
pub fn distance_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    if a == b {
        return p.distance(a);
    }
    segment_distance(p, a, b)
}

pub struct Map {
    pub name: &'static str,
    pub waypoints: Vec<Vec2>,
    /// Where the spawn portal is drawn.
    pub portal: Vec2,
    /// Where the castle is drawn.
    pub base: Vec2,
    /// Grass tufts for decoration: position, radius, shade.
    pub decor: Vec<(Vec2, f32, f32)>,
}

impl Map {
    pub fn new(index: usize) -> Self {
        let (name, waypoints, portal, base) = match index % MAP_COUNT {
            0 => (
                "Sketch",
                SKETCH.iter().map(|&(x, y)| from_sketch(x, y)).collect(),
                from_sketch(14.0, 236.0),
                from_sketch(466.0, 482.0),
            ),
            _ => (
                "Zigzag",
                ZIGZAG.iter().map(|&(c, r)| grid_center(c, r)).collect(),
                grid_center(0, 2),
                grid_center(19, 5),
            ),
        };
        let mut map = Self {
            name,
            waypoints,
            portal,
            base,
            decor: Vec::new(),
        };
        map.decor = map.make_decor();
        map
    }

    pub fn distance_to_path(&self, p: Vec2) -> f32 {
        self.waypoints
            .windows(2)
            .map(|w| segment_distance(p, w[0], w[1]))
            .fold(f32::INFINITY, f32::min)
    }

    /// Deterministic scattering of grass tufts that avoid the path.
    fn make_decor(&self) -> Vec<(Vec2, f32, f32)> {
        let mut seed: u32 = 0x9e37_79b9;
        let mut rand = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32
        };
        let mut decor = Vec::new();
        for _ in 0..260 {
            let p = vec2(MAP_X + rand() * MAP_W, MAP_Y + rand() * MAP_H);
            let r = 3.0 + rand() * 9.0;
            let shade = rand();
            if self.distance_to_path(p) > PATH_WIDTH / 2.0 + r + 4.0 {
                decor.push((p, r, shade));
            }
        }
        decor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sketch_enters_left_and_exits_bottom() {
        let map = Map::new(0);
        let first = map.waypoints[0];
        let last = *map.waypoints.last().unwrap();
        assert!(first.x < MAP_X && !in_map(first, 0.0));
        assert!(last.y > MAP_Y + MAP_H);
    }

    #[test]
    fn path_distance() {
        let map = Map::new(0);
        assert!(map.distance_to_path(map.portal) < 1.0);
        assert!(map.distance_to_path(from_sketch(300.0, 160.0)) > PATH_WIDTH);
    }
}
