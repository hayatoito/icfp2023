pub use anyhow::{bail, ensure, Context, Result};
pub use approx::*;
pub use log::*;
pub use ordered_float::OrderedFloat;
pub use rand::rngs::StdRng;
pub use rand::Rng;
pub use rand::SeedableRng;
pub use serde::{Deserialize, Serialize};
pub use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
pub use std::io::Write;
pub use std::ops::Deref;
pub use std::ops::Index;
pub use std::ops::IndexMut;
pub use std::ops::Range;
pub use std::path::{Path, PathBuf};

pub use indicatif::ProgressBar;

pub fn project_path(relative_path: impl AsRef<Path>) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push(relative_path);
    path
}

pub fn read_from(relative_path: impl AsRef<Path>) -> Result<String> {
    let path = project_path(relative_path);
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_to(relative_path: impl AsRef<Path>, content: &str) -> Result<()> {
    let path = project_path(relative_path);
    std::fs::create_dir_all(path.parent().unwrap())?;
    Ok(std::fs::write(path, content)?)
}

pub type Score = f64;
pub type Coord = f64;
pub type Instrument = usize;
pub type Volume = Score;

pub const MUSICIAN_RADIUS: Coord = 10.0;
pub const MUSICIAN_RADIUS_2: Coord = MUSICIAN_RADIUS * MUSICIAN_RADIUS;

pub const BLOCK_RADIUS: Coord = 5.0;
pub const BLOCK_RADIUS_2: Coord = BLOCK_RADIUS * BLOCK_RADIUS;

pub const EPS: Coord = 1.0e-10;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, derive_more::Display)]
#[display(fmt = "({}, {})", x, y)]
pub struct Point {
    pub x: Coord,
    pub y: Coord,
}

impl Point {
    pub fn new(x: Coord, y: Coord) -> Self {
        Point { x, y }
    }

    pub fn distance_squared(&self, other: Point) -> Coord {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    pub fn distance(&self, other: Point) -> Coord {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx.hypot(dy)
    }
}

// ChatGPT
pub fn is_line_circle_intersect(p1: Point, p2: Point, center: Point, radius: Coord) -> bool {
    let distance_squared = point_to_segment_distance_squared(center, (p1, p2));
    distance_squared < radius * radius
}

pub fn point_to_segment_distance_squared(p: Point, (p1, p2): (Point, Point)) -> Coord {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let d_squared = dx * dx + dy * dy;
    let a = ((p.x - p1.x) * dx + (p.y - p1.y) * dy) / d_squared;
    let a = a.clamp(0.0, 1.0);
    let closest_x = p1.x + a * dx;
    let closest_y = p1.y + a * dy;
    let distance_squared = (closest_x - p.x).powi(2) + (closest_y - p.y).powi(2);
    distance_squared
}

pub fn inst_cnt(musicians: &[Instrument]) -> HashMap<Instrument, usize> {
    // # of instruments
    let mut cnt = HashMap::new();
    for inst in musicians.iter() {
        *cnt.entry(*inst).or_insert(0) += 1;
    }
    cnt
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn distance() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(4.0, 6.0);
        assert_eq!(p1.distance(p2), 5.0);
        assert_eq!(p1.distance_squared(p2), 25.0);
    }

    #[test]
    fn is_line_circle_intersect_test() {
        let radius = 5.0;
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 0.0);
        assert!(is_line_circle_intersect(
            p1,
            p2,
            Point::new(5.0, 0.0),
            radius
        ));

        assert!(is_line_circle_intersect(
            p1,
            p2,
            Point::new(5.0, 4.9),
            radius
        ));

        assert!(!is_line_circle_intersect(
            p1,
            p2,
            Point::new(5.0, 5.0),
            radius
        ));

        assert!(!is_line_circle_intersect(
            p1,
            p2,
            Point::new(5.0, 5.1),
            radius
        ));

        assert!(!is_line_circle_intersect(
            p1,
            p2,
            Point::new(15.0, 0.0),
            radius
        ));

        assert!(!is_line_circle_intersect(
            p1,
            p2,
            Point::new(15.0, 0.1),
            radius
        ));

        assert!(is_line_circle_intersect(
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(0.0, 0.0),
            radius
        ));

        assert!(is_line_circle_intersect(
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(0.0, 5.0),
            radius
        ));

        assert!(is_line_circle_intersect(
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(0.0, 7.0),
            radius
        ));

        assert!(!is_line_circle_intersect(
            Point::new(0.0, 0.0),
            Point::new(5.0, 5.0),
            Point::new(0.0, 7.1),
            radius
        ));
    }
}
