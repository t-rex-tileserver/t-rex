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

/// Generic Geometry Data Type
#[derive(Debug)]
pub enum GeometryType {
    Point(Point),
    LineString(LineString),
    Polygon(Polygon),
    MultiPoint(MultiPoint),
    MultiLineString(MultiLineString),
    MultiPolygon(MultiPolygon),
    GeometryCollection(GeometryCollection)
}

impl Clone for GeometryType {
    fn clone(&self) -> Self {
        match self {
            &GeometryType::Point(ref p) => GeometryType::Point(Point::new(p.x, p.y, None)),
            _ => panic!("Not implemented yet") // TODO: either implement other types or don't clone (FeatureStruct)...
        }
    }
}

impl GeometryType {
    pub fn is_empty(&self) -> bool {
        match self {
            &GeometryType::LineString(ref p) => p.points.len() == 0,
            &GeometryType::Polygon(ref p) => p.rings.len() == 0,
            &GeometryType::MultiPoint(ref p) => p.points.len() == 0,
            &GeometryType::MultiLineString(ref p) => p.lines.len() == 0,
            &GeometryType::MultiPolygon(ref p) => p.polygons.len() == 0,
            _ => false
        }
    }
}

#[cfg(test)]
impl GeometryType {
    pub fn new_point(x: f64, y: f64) -> GeometryType {
        GeometryType::Point(Point::new(x, y, None))
    }
}

#[test]
fn test_geom_creation() {
    let _ : GeometryType = GeometryType::Point(ewkb::Point::new(960000.0, 6002729.0, Some(3857)));
    let _ : GeometryType = GeometryType::Point(Point::new(960000.0, 6002729.0, Some(3857)));
    let g3 = GeometryType::new_point(960000.0, 6002729.0);
    let p = match g3 { GeometryType::Point(p) => p, _ => panic!() };
    assert_eq!(p.x, 960000.0);
}
