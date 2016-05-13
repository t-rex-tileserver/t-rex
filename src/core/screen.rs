//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! Geometry types in screen coordinates

use std::vec::Vec;


#[derive(PartialEq,Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32
}

#[derive(PartialEq,Debug)]
pub struct MultiPoint {
    pub points: Vec<Point>
}

#[derive(PartialEq,Debug)]
pub struct Linestring {
    pub points: Vec<Point>
}
