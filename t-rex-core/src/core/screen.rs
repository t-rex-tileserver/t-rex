//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! Geometry types in screen coordinates

use std::vec::Vec;

#[derive(PartialEq, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn origin() -> Point {
        Point { x: 0, y: 0 }
    }
}

#[derive(PartialEq, Debug)]
pub struct MultiPoint {
    pub points: Vec<Point>,
}

#[derive(PartialEq, Debug)]
pub struct LineString {
    pub points: Vec<Point>,
}

#[derive(PartialEq, Debug)]
pub struct MultiLineString {
    pub lines: Vec<LineString>,
}

#[derive(PartialEq, Debug)]
pub struct Polygon {
    pub rings: Vec<LineString>,
}

#[derive(PartialEq, Debug)]
pub struct MultiPolygon {
    pub polygons: Vec<Polygon>,
}
