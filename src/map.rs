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

/// The default map, traced from a 512x512 sketch and straightened so every
/// segment is horizontal or vertical. It enters from the left, loops back
/// over itself twice and ends at the castle near the bottom.
const SKETCH: [(f32, f32); 13] = [
    (-24.0, 236.0),
    (189.0, 236.0),
    (189.0, 86.0),
    // Moved right of the hand-drawn x=80 so the strip beside x=42 is wide
    // enough for towers.
    (92.0, 86.0),
    (92.0, 395.0),
    (261.0, 395.0),
    (261.0, 297.0),
    (42.0, 297.0),
    (42.0, 464.0),
    (362.0, 464.0),
    (362.0, 84.0),
    (464.0, 84.0),
    (464.0, 478.0),
];

/// The castle map's rug, in 48px grid cells. Longer than the original
/// zigzag, with every parallel stretch far enough apart for towers.
const ZIGZAG: [(i32, i32); 10] = [
    (-1, 2),
    (4, 2),
    (4, 10),
    (9, 10),
    (9, 3),
    (14, 3),
    (14, 10),
    (17, 10),
    (17, 6),
    (19, 6),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapTheme {
    /// Grass, trees and rocks.
    Meadow,
    /// Stone floor, a rug for the track, torches and pillars.
    Castle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Tree,
    Rock,
    Torch,
    Pillar,
}

/// Scenery that towers can't be placed on.
#[derive(Clone, Copy, Debug)]
pub struct Prop {
    pub kind: PropKind,
    pub pos: Vec2,
    pub radius: f32,
    /// Per-prop random value for variety (size, flicker phase...).
    pub seed: f32,
}

/// Small deterministic xorshift generator, so maps look the same every time.
struct Rng(u32);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 as f32 / u32::MAX as f32
    }
}

#[derive(Default)]
pub struct Map {
    pub name: &'static str,
    pub description: &'static str,
    pub theme: Option<MapTheme>,
    pub waypoints: Vec<Vec2>,
    /// Where the spawn portal is drawn.
    pub portal: Vec2,
    /// The castle (or throne); it sits on the last waypoint, so enemies that
    /// reach it damage it.
    pub base: Vec2,
    /// Non-blocking ground detail: position, radius, shade.
    pub decor: Vec<(Vec2, f32, f32)>,
    pub props: Vec<Prop>,
}

impl Map {
    pub fn new(index: usize) -> Self {
        let (name, description, theme, waypoints, portal, base) = match index % MAP_COUNT {
            0 => (
                "Sketch",
                "Long meadow path. Normal",
                MapTheme::Meadow,
                SKETCH.iter().map(|&(x, y)| from_sketch(x, y)).collect(),
                from_sketch(14.0, 236.0),
                from_sketch(464.0, 478.0),
            ),
            _ => (
                "Castle",
                "Short rug to the throne. Hard",
                MapTheme::Castle,
                ZIGZAG.iter().map(|&(c, r)| grid_center(c, r)).collect(),
                grid_center(0, 2),
                grid_center(19, 6),
            ),
        };
        let mut map = Self {
            name,
            description,
            theme: Some(theme),
            waypoints,
            portal,
            base,
            decor: Vec::new(),
            props: Vec::new(),
        };
        map.decor = map.make_decor();
        map.props = match theme {
            MapTheme::Meadow => map.meadow_props(),
            MapTheme::Castle => map.castle_props(),
        };
        map
    }

    pub fn theme(&self) -> MapTheme {
        self.theme.unwrap_or(MapTheme::Meadow)
    }

    pub fn distance_to_path(&self, p: Vec2) -> f32 {
        self.waypoints
            .windows(2)
            .map(|w| segment_distance(p, w[0], w[1]))
            .fold(f32::INFINITY, f32::min)
    }

    /// Whether a circle at `pos` would overlap any scenery.
    pub fn blocked(&self, pos: Vec2, radius: f32) -> bool {
        self.props
            .iter()
            .any(|p| p.pos.distance(pos) < p.radius + radius)
    }

    fn free_spot(&self, p: Vec2, path_gap: f32, props: &[Prop], spacing: f32) -> bool {
        self.distance_to_path(p) > PATH_WIDTH / 2.0 + path_gap
            && p.distance(self.base) > 70.0
            && p.distance(self.portal) > 50.0
            && props.iter().all(|o| o.pos.distance(p) > spacing)
    }

    /// Deterministic scattering of ground detail that avoids the path.
    fn make_decor(&self) -> Vec<(Vec2, f32, f32)> {
        let mut rng = Rng(0x9e37_79b9);
        let mut decor = Vec::new();
        for _ in 0..260 {
            let p = vec2(MAP_X + rng.next() * MAP_W, MAP_Y + rng.next() * MAP_H);
            let r = 3.0 + rng.next() * 9.0;
            let shade = rng.next();
            if self.distance_to_path(p) > PATH_WIDTH / 2.0 + r + 4.0 {
                decor.push((p, r, shade));
            }
        }
        decor
    }

    /// Trees around the edges of the meadow and a few rocks.
    fn meadow_props(&self) -> Vec<Prop> {
        let mut rng = Rng(0x2545_f491);
        let mut props: Vec<Prop> = Vec::new();
        for _ in 0..400 {
            let p = vec2(MAP_X + rng.next() * MAP_W, MAP_Y + rng.next() * MAP_H);
            let edge = p.x < MAP_X + 60.0
                || p.x > MAP_X + MAP_W - 60.0
                || p.y < MAP_Y + 50.0
                || p.y > MAP_Y + MAP_H - 50.0;
            let trees = props.iter().filter(|p| p.kind == PropKind::Tree).count();
            if edge && trees < 16 && self.free_spot(p, 30.0, &props, 56.0) {
                props.push(Prop {
                    kind: PropKind::Tree,
                    pos: p,
                    radius: 20.0,
                    seed: rng.next(),
                });
            }
        }
        for _ in 0..200 {
            let p = vec2(MAP_X + rng.next() * MAP_W, MAP_Y + rng.next() * MAP_H);
            let rocks = props.iter().filter(|p| p.kind == PropKind::Rock).count();
            if rocks < 9 && in_map(p, 20.0) && self.free_spot(p, 14.0, &props, 90.0) {
                props.push(Prop {
                    kind: PropKind::Rock,
                    pos: p,
                    radius: 9.0,
                    seed: rng.next(),
                });
            }
        }
        props
    }

    /// Torches lining both sides of the rug, plus pillars across the hall.
    fn castle_props(&self) -> Vec<Prop> {
        let mut rng = Rng(0x68e3_1da4);
        let mut props: Vec<Prop> = Vec::new();
        let mut side = 1.0;
        for w in self.waypoints.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = a.distance(b);
            let dir = (b - a) / len;
            let normal = vec2(-dir.y, dir.x);
            let mut d = 70.0;
            while d < len - 30.0 {
                let p = a + dir * d + normal * side * (PATH_WIDTH / 2.0 + 16.0);
                side = -side;
                d += 140.0;
                if in_map(p, 10.0) && self.free_spot(p, 8.0, &props, 60.0) {
                    props.push(Prop {
                        kind: PropKind::Torch,
                        pos: p,
                        radius: 9.0,
                        seed: rng.next(),
                    });
                }
            }
        }
        let mut y = MAP_Y + 96.0;
        while y < MAP_Y + MAP_H {
            let mut x = MAP_X + 72.0;
            while x < MAP_X + MAP_W {
                let p = vec2(x, y);
                if self.free_spot(p, 34.0, &props, 70.0) {
                    props.push(Prop {
                        kind: PropKind::Pillar,
                        pos: p,
                        radius: 15.0,
                        seed: rng.next(),
                    });
                }
                x += 240.0;
            }
            y += 240.0;
        }
        props
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_start_off_map_and_end_at_the_castle() {
        for i in 0..MAP_COUNT {
            let map = Map::new(i);
            assert!(!in_map(map.waypoints[0], 0.0));
            assert_eq!(*map.waypoints.last().unwrap(), map.base);
        }
    }

    #[test]
    fn segments_are_straight() {
        for i in 0..MAP_COUNT {
            for w in Map::new(i).waypoints.windows(2) {
                assert!(w[0].x == w[1].x || w[0].y == w[1].y, "{w:?}");
            }
        }
    }

    #[test]
    fn castle_map_has_torches_and_props_block_placement() {
        let map = Map::new(1);
        assert_eq!(map.theme(), MapTheme::Castle);
        let torch = map
            .props
            .iter()
            .find(|p| p.kind == PropKind::Torch)
            .unwrap();
        assert!(map.blocked(torch.pos, TOWER_RADIUS));
        assert!(
            map.props
                .iter()
                .all(|p| map.distance_to_path(p.pos) > PATH_WIDTH / 2.0)
        );
    }

    /// Parallel stretches of track (and the map edges) must either touch or
    /// leave room for a tower; nothing in between that nobody can use.
    #[test]
    fn gaps_between_paths_fit_towers() {
        let fits = PATH_WIDTH + 2.0 * TOWER_RADIUS + 4.0;
        for i in 0..MAP_COUNT {
            let map = Map::new(i);
            let segs: Vec<(Vec2, Vec2)> = map.waypoints.windows(2).map(|w| (w[0], w[1])).collect();
            for (a, &(a0, a1)) in segs.iter().enumerate() {
                for &(b0, b1) in segs.iter().skip(a + 2) {
                    let both_vertical = a0.x == a1.x && b0.x == b1.x;
                    let both_horizontal = a0.y == a1.y && b0.y == b1.y;
                    let (gap, overlap) = if both_vertical {
                        let lo = a0.y.min(a1.y).max(b0.y.min(b1.y));
                        let hi = a0.y.max(a1.y).min(b0.y.max(b1.y));
                        ((a0.x - b0.x).abs(), hi > lo)
                    } else if both_horizontal {
                        let lo = a0.x.min(a1.x).max(b0.x.min(b1.x));
                        let hi = a0.x.max(a1.x).min(b0.x.max(b1.x));
                        ((a0.y - b0.y).abs(), hi > lo)
                    } else {
                        continue;
                    };
                    assert!(
                        !overlap || gap < PATH_WIDTH || gap >= fits,
                        "{}: paths {gap}px apart leave a strip too thin for towers",
                        map.name
                    );
                }
                // Same for the space between the track and the map edge.
                let edge_fits = PATH_WIDTH / 2.0 + 2.0 * TOWER_RADIUS + 2.0;
                let edge_gaps = if a0.x == a1.x {
                    [a0.x - MAP_X, MAP_X + MAP_W - a0.x]
                } else {
                    [a0.y - MAP_Y, MAP_Y + MAP_H - a0.y]
                };
                for e in edge_gaps {
                    assert!(
                        e <= PATH_WIDTH / 2.0 + 4.0 || e >= edge_fits,
                        "{}: path {e}px from the edge leaves a strip too thin for towers",
                        map.name
                    );
                }
            }
        }
    }

    #[test]
    fn castle_map_is_longer_than_before() {
        let map = Map::new(1);
        let length: f32 = map.waypoints.windows(2).map(|w| w[0].distance(w[1])).sum();
        // The original zigzag was 36 tiles long.
        assert!(length > 40.0 * TILE, "{}", length / TILE);
    }

    #[test]
    fn path_distance() {
        let map = Map::new(0);
        assert!(map.distance_to_path(map.portal) < 1.0);
        assert!(map.distance_to_path(from_sketch(300.0, 160.0)) > PATH_WIDTH);
    }
}
