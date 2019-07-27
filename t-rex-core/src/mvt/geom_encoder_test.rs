//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::screen;
use crate::mvt::geom_encoder::EncodableGeom;

#[test]
fn test_geom_encoding() {
    let point = screen::Point { x: 25, y: 17 };
    assert_eq!(point.encode().0, &[9, 50, 34]);

    let multipoint = screen::MultiPoint {
        points: vec![screen::Point { x: 5, y: 7 }, screen::Point { x: 3, y: 2 }],
    };
    assert_eq!(multipoint.encode().0, &[17, 10, 14, 3, 9]);

    let linestring = screen::LineString {
        points: vec![
            screen::Point { x: 2, y: 2 },
            screen::Point { x: 2, y: 10 },
            screen::Point { x: 10, y: 10 },
        ],
    };
    assert_eq!(linestring.encode().0, &[9, 4, 4, 18, 0, 16, 16, 0]);

    let multilinestring = screen::MultiLineString {
        lines: vec![
            screen::LineString {
                points: vec![
                    screen::Point { x: 2, y: 2 },
                    screen::Point { x: 2, y: 10 },
                    screen::Point { x: 10, y: 10 },
                ],
            },
            screen::LineString {
                points: vec![screen::Point { x: 1, y: 1 }, screen::Point { x: 3, y: 5 }],
            },
        ],
    };
    assert_eq!(
        multilinestring.encode().0,
        &[9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8]
    );

    let polygon = screen::Polygon {
        rings: vec![screen::LineString {
            points: vec![
                screen::Point { x: 3, y: 6 },
                screen::Point { x: 8, y: 12 },
                screen::Point { x: 20, y: 34 },
                screen::Point { x: 3, y: 6 },
            ],
        }],
    };
    assert_eq!(polygon.encode().0, &[9, 6, 12, 18, 10, 12, 24, 44, 15]);

    let multipolygon = screen::MultiPolygon {
        polygons: vec![
            screen::Polygon {
                rings: vec![screen::LineString {
                    points: vec![
                        screen::Point { x: 0, y: 0 },
                        screen::Point { x: 10, y: 0 },
                        screen::Point { x: 10, y: 10 },
                        screen::Point { x: 0, y: 10 },
                        screen::Point { x: 0, y: 0 },
                    ],
                }],
            },
            screen::Polygon {
                rings: vec![
                    screen::LineString {
                        points: vec![
                            screen::Point { x: 11, y: 11 },
                            screen::Point { x: 20, y: 11 },
                            screen::Point { x: 20, y: 20 },
                            screen::Point { x: 11, y: 20 },
                            screen::Point { x: 11, y: 20 },
                            screen::Point { x: 11, y: 11 },
                        ],
                    },
                    screen::LineString {
                        points: vec![
                            screen::Point { x: 13, y: 13 },
                            screen::Point { x: 13, y: 17 },
                            screen::Point { x: 17, y: 17 },
                            screen::Point { x: 17, y: 13 },
                            screen::Point { x: 13, y: 13 },
                        ],
                    },
                ],
            },
        ],
    };
    let expected = [
        9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 34, 18, 0, 0, 18, 17, 0, 0, 0, 15, 9, 4,
        13, 26, 0, 8, 8, 0, 0, 7, 15,
    ];
    assert_eq!(multipolygon.encode().0, &expected[0..35]);
}

#[test]
fn test_overflow() {
    use std::i32;
    use std::u32;

    assert_eq!(i32::MIN, -2147483648);
    assert_eq!(i32::MAX, 2147483647);
    assert_eq!(u32::MAX, 4294967295);

    let multipoint = screen::MultiPoint {
        points: vec![
            screen::Point { x: 5, y: 7 },
            screen::Point {
                x: i32::MIN,
                y: i32::MIN,
            },
        ],
    };
    assert_eq!(multipoint.encode().0, &[17, 10, 14, u32::MAX, u32::MAX]);

    let multipoint = screen::MultiPoint {
        points: vec![
            screen::Point { x: -5, y: -10 },
            screen::Point {
                x: i32::MAX,
                y: i32::MAX,
            },
        ],
    };
    assert_eq!(
        multipoint.encode().0,
        &[17, 9, 19, u32::MAX - 1, u32::MAX - 1]
    );
}
