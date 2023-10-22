//! Calculates the radius and diameter of a set of points
//!
//! The radius of a set of points is defined as the smallest distance such that there exists some
//! point in the set which is no further than that distance to any point in the set.
//!
//! The diameter of a set of points is defined as the smallest distance such that no two points in
//! the set are further from each other than that distance.
//!
//! In mathematical terms, where `d(i, j)` is the distance from point `i` to point `j`:
//! - `radius = points.flat_map(|i| points.map(|j| d(i, j)).max()).min()`
//! - `diameter = points.flat_map(|i| points.map(|j| d(i, j)).max()).max()`

use geo::{GeodesicDistance, Point};

pub fn radius_and_diameter(points: &[Point]) -> (f64, f64) {
    points
        .iter()
        .flat_map(|p1| {
            points
                .iter()
                .map(|p2| p1.geodesic_distance(p2))
                .reduce(f64::max)
        })
        .fold((0.0, 0.0), |(min, max), dist| (min.min(dist), max.max(dist)))
}
