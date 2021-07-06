//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use postgis::ewkb;

// Aliases for rust-postgis geometry types
pub type Point = ewkb::Point;
pub type LineString = ewkb::LineString;
pub type Polygon = ewkb::Polygon;
pub type MultiPoint = ewkb::MultiPoint;
pub type MultiLineString = ewkb::MultiLineString;
pub type MultiPolygon = ewkb::MultiPolygon;
pub type GeometryCollection = ewkb::GeometryCollection;
pub type Geometry = ewkb::Geometry;

/// Generic Geometry Data Type
#[derive(Debug)]
pub enum GeometryType {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection),
    Geometry(Geometry),
}

impl GeometryType {
    pub fn is_empty(&self) -> bool {
        match self {
            &GeometryType::LineString(ref p) => p.points.len() == 0,
            &GeometryType::Polygon(ref p) => p.rings.len() == 0,
            &GeometryType::MultiPoint(ref p) => p.points.len() == 0,
            &GeometryType::MultiLineString(ref p) => p.lines.len() == 0,
            &GeometryType::MultiPolygon(ref p) => p.polygons.len() == 0,
            _ => false,
        }
    }
}
