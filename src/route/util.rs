// Copyright 2020 David Li
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Route guide utilities.

use crate::{FeatureDatabase, Point, Rectangle};
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

const COORD_FACTOR: f64 = 1e7;

/// Indicates whether the given point is in the range of the given rectangle.
#[inline]
pub fn in_range(point: &Point, rect: &Rectangle) -> bool {
    use std::cmp::{max, min};

    let lo = rect.lo.as_ref().unwrap();
    let hi = rect.hi.as_ref().unwrap();

    let left = min(lo.longitude, hi.longitude);
    let right = max(lo.longitude, hi.longitude);
    let top = max(lo.latitude, hi.latitude);
    let bottom = min(lo.latitude, hi.latitude);

    let lat = point.latitude;
    let lon = point.longitude;

    if lon >= left && lon <= right && lat >= bottom && lat <= top {
        true
    } else {
        false
    }
}

/// Gets the latitude for the given point.
#[inline]
fn get_latitude(location: &Point) -> f64 {
    location.latitude as f64 / COORD_FACTOR
}

/// Gets the longitude for the given point.
#[inline]
fn get_longitude(location: &Point) -> f64 {
    location.longitude as f64 / COORD_FACTOR
}

/// Calculates distance between two points.
#[inline]
pub fn calc_distance(start: &Point, end: &Point) -> i32 {
    const R: i32 = 6371000; // earth radius in meters

    let lat1 = get_latitude(start).to_radians();
    let lat2 = get_latitude(end).to_radians();
    let lon1 = get_longitude(start).to_radians();
    let lon2 = get_longitude(end).to_radians();

    let delta_lat = lat2 - lat1;
    let delta_lon = lon2 - lon1;

    let a = (delta_lat / 2f64).sin() * (delta_lat / 2f64).sin()
        + lat1.cos() * lat2.cos() * (delta_lon / 2f64).sin() * (delta_lon / 2f64).sin();
    let c = 2f64 * a.sqrt().atan2((1f64 - a).sqrt());
    let distance = R as f64 * c;
    distance as i32
}

impl Hash for Point {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.longitude.hash(state);
        self.longitude.hash(state);
    }
}

impl Eq for Point {}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({}, {})", get_latitude(self), get_longitude(self))
    }
}

#[derive(Debug, Deserialize)]
struct DB {
    feature: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    name: String,
    location: Location,
}

#[derive(Debug, Deserialize)]
struct Location {
    latitude: i32,
    longitude: i32,
}

/// Gets the default features file.
#[inline]
fn get_default_features_file() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR");
    let path = PathBuf::from(dir).join("data/route_guide_db.json");
    assert!(path.exists());
    path
}

/// Parses the JSON input file containing the list of features.
#[inline]
pub fn load_database() -> FeatureDatabase {
    let file = get_default_features_file();
    let file = std::fs::File::open(file).unwrap();
    let db: DB = serde_json::from_reader(file).unwrap();

    let feature = db
        .feature
        .into_iter()
        .map(|f| crate::routeguide::Feature {
            name: f.name,
            location: Some(crate::routeguide::Point {
                longitude: f.location.longitude,
                latitude: f.location.latitude,
            }),
        })
        .collect();

    FeatureDatabase { feature }
}
