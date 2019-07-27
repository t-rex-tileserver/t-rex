//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::geom::{GeometryType, Point};
use postgis::ewkb;

#[cfg(test)]
impl GeometryType {
    pub fn new_point(x: f64, y: f64) -> GeometryType {
        GeometryType::Point(Point::new(x, y, None))
    }
}

#[test]
fn test_geom_creation() {
    let _: GeometryType = GeometryType::Point(ewkb::Point::new(960000.0, 6002729.0, Some(3857)));
    let _: GeometryType = GeometryType::Point(Point::new(960000.0, 6002729.0, Some(3857)));
    let g3 = GeometryType::new_point(960000.0, 6002729.0);
    let p = match g3 {
        GeometryType::Point(p) => p,
        _ => panic!(),
    };
    assert_eq!(p.x, 960000.0);
}
