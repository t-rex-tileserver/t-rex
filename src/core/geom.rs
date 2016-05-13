//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use postgis;


#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
pub enum EPSG_3857 {}

impl postgis::SRID for EPSG_3857 {
    fn as_srid() -> Option<i32> { Some(3857) }
}

// Aliases for rust-postgis geometry types
// To support arbitrary SRIDs we will have to define our own geometry types
pub type Point = postgis::Point<EPSG_3857>;
pub type LineString = postgis::LineString<postgis::Point<EPSG_3857>>;
pub type Polygon = postgis::Polygon<postgis::Point<EPSG_3857>>;
pub type MultiPoint = postgis::MultiPoint<postgis::Point<EPSG_3857>>;
pub type MultiLineString = postgis::MultiLineString<postgis::Point<EPSG_3857>>;
pub type MultiPolygon = postgis::MultiPolygon<postgis::Point<EPSG_3857>>;
pub type GeometryCollection = postgis::GeometryCollection<postgis::Point<EPSG_3857>>;

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
            &GeometryType::Point(ref p) => GeometryType::Point(Point::new(p.x, p.y)),
            _ => panic!("Not implemented yet") // TODO: either implement other types or don't clone (FeatureStruct)...
        }
    }
}

#[cfg(test)]
impl GeometryType {
    pub fn new_point(x: f64, y: f64) -> GeometryType {
        GeometryType::Point(Point::new(x, y))
    }
}

#[test]
fn test_geom_creation() {
    let g1 : GeometryType = GeometryType::Point(postgis::Point::<EPSG_3857>::new(960000.0, 6002729.0));
    let g2 : GeometryType = GeometryType::Point(Point::new(960000.0, 6002729.0));
    let g3 = GeometryType::new_point(960000.0, 6002729.0);
    let p = match g3 { GeometryType::Point(p) => p, _ => panic!() };
    assert_eq!(p.x, 960000.0);
}
