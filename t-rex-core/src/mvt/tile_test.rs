//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;
use core::feature::FeatureAttrValType;
use core::feature::{FeatureStruct, FeatureAttr};
use core::grid::Extent;
use core::geom::GeometryType;
use core::geom;
use core::screen;
use mvt::vector_tile;
use mvt::geom_encoder::EncodableGeom;
use mvt::tile::{Tile, ScreenGeom};
use std::fs::File;


#[test]
fn test_point_to_screen_coords() {
    use std::f64;
    use std::i32;

    //let zh_mercator = geom::Point::new(949398.0, 6002729.0, Some(3857));
    let zh_mercator = geom::Point::new(960000.0, 6002729.0, Some(3857));
    //let zh_wgs84 = postgis::Point::new(47.3703149, 8.5285874, Some(4326));
    let tile_extent = Extent {
        minx: 958826.08,
        miny: 5987771.04,
        maxx: 978393.96,
        maxy: 6007338.92,
    };
    let screen_pt = screen::Point::from_geom(&tile_extent, false, 4096, &zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    assert_eq!(screen_pt.encode().vec(), &[9, 490, 6262]);

    //overflow
    let point = geom::Point::new(960000.0, f64::MAX, Some(3857));
    let screen_pt = screen::Point::from_geom(&tile_extent, false, 4096, &point);
    assert_eq!(screen_pt,
               screen::Point {
                   x: 245,
                   y: i32::MIN,
               });
    let screen_pt = screen::Point::from_geom(&tile_extent, true, 4096, &point);
    assert_eq!(screen_pt,
               screen::Point {
                   x: 245,
                   y: i32::MAX,
               });
}


#[test]
fn test_tile_values() {
    let mut value = vector_tile::Tile_Value::new();
    assert_eq!(value, Default::default());
    assert!(!value.has_string_value());
    value.set_string_value(String::from("Hello, world!"));
    println!("{:?}", value);
    assert!(value.has_string_value());
    assert_eq!(value.get_string_value(), String::from("Hello, world!"));
}

#[test]
fn test_read_from_file() {
    // Command line decoding:
    // protoc --decode=vector_tile.Tile src/mvt/vector_tile.proto <src/test/tile.pbf
    let mut f = File::open("../t-rex-service/src/test/tile.pbf").unwrap();
    let tile = Tile::read_from(&mut f).unwrap();
    println!("{:#?}", tile);
    let ref layer = tile.get_layers()[0];
    assert_eq!(layer.get_name(), "roads");
    let ref feature = layer.get_features()[1];
    assert_eq!(feature.get_field_type(),
               vector_tile::Tile_GeomType::POLYGON);
    let ref geometry = feature.get_geometry();
    assert_eq!(geometry,
               &[9, 8236, 4926, 34, 9, 24, 37, 21, 10, 7, 4, 19, 15]);
}


// https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
#[cfg(test)]
const TILE_EXAMPLE: &'static str = r#"Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some("points"),
            features: [
                Tile_Feature {
                    id: Some(
                        1
                    ),
                    tags: [
                        0,
                        0,
                        1,
                        0,
                        2,
                        1
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        490,
                        6262
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Feature {
                    id: Some(
                        2
                    ),
                    tags: [
                        0,
                        2,
                        2,
                        3
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        490,
                        6262
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            keys: [
                "hello",
                "h",
                "count"
            ],
            values: [
                Tile_Value {
                    string_value: Some("world"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        1.23
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: Some("again"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        2
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell {
                value: 0
            }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell {
        value: 0
    }
}"#;

#[test]
fn test_build_mvt() {
    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
    let mut mvt_tile = vector_tile::Tile::new();

    let mut mvt_layer = vector_tile::Tile_Layer::new();
    mvt_layer.set_version(2);
    mvt_layer.set_name(String::from("points"));
    mvt_layer.set_extent(4096);

    let mut mvt_feature = vector_tile::Tile_Feature::new();
    mvt_feature.set_id(1);
    mvt_feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    mvt_feature.set_geometry([9, 490, 6262].to_vec());

    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("world"));
    Tile::add_feature_attribute(&mut mvt_layer,
                                &mut mvt_feature,
                                String::from("hello"),
                                mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("world"));
    Tile::add_feature_attribute(&mut mvt_layer,
                                &mut mvt_feature,
                                String::from("h"),
                                mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_double_value(1.23);
    Tile::add_feature_attribute(&mut mvt_layer,
                                &mut mvt_feature,
                                String::from("count"),
                                mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_feature = vector_tile::Tile_Feature::new();
    mvt_feature.set_id(2);
    mvt_feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    mvt_feature.set_geometry([9, 490, 6262].to_vec());

    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("again"));
    Tile::add_feature_attribute(&mut mvt_layer,
                                &mut mvt_feature,
                                String::from("hello"),
                                mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_int_value(2);
    Tile::add_feature_attribute(&mut mvt_layer,
                                &mut mvt_feature,
                                String::from("count"),
                                mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_tile.mut_layers().push(mvt_layer);
    println!("{:#?}", mvt_tile);
    assert_eq!(TILE_EXAMPLE, &*format!("{:#?}", mvt_tile));
}

#[test]
fn test_build_mvt_with_helpers() {
    use std::env;

    let extent = Extent {
        minx: 958826.08,
        miny: 5987771.04,
        maxx: 978393.96,
        maxy: 6007338.92,
    };
    let mut tile = Tile::new(&extent, 4096, false);
    let layer = Layer::new("points");
    let mut mvt_layer = tile.new_layer(&layer);

    let geom: GeometryType = GeometryType::Point(geom::Point::new(960000.0, 6002729.0, Some(3857)));
    let feature = FeatureStruct {
        fid: Some(1),
        attributes: vec![FeatureAttr {
                             key: String::from("hello"),
                             value: FeatureAttrValType::String(String::from("world")),
                         },
                         FeatureAttr {
                             key: String::from("h"),
                             value: FeatureAttrValType::String(String::from("world")),
                         },
                         FeatureAttr {
                             key: String::from("count"),
                             value: FeatureAttrValType::Double(1.23),
                         }],
        geometry: geom,
    };
    tile.add_feature(&mut mvt_layer, &feature);

    let geom: GeometryType = GeometryType::Point(geom::Point::new(960000.0, 6002729.0, Some(3857)));
    let feature = FeatureStruct {
        fid: Some(2),
        attributes: vec![FeatureAttr {
                             key: String::from("hello"),
                             value: FeatureAttrValType::String(String::from("again")),
                         },
                         FeatureAttr {
                             key: String::from("count"),
                             value: FeatureAttrValType::Int(2),
                         }],
        geometry: geom,
    };
    tile.add_feature(&mut mvt_layer, &feature);

    tile.add_layer(mvt_layer);
    println!("{:#?}", tile.mvt_tile);
    assert_eq!(TILE_EXAMPLE, &*format!("{:#?}", tile.mvt_tile));

    let mut path = env::temp_dir();
    path.push("out.pbf");
    tile.to_file(&format!("{}", &path.display()));
}
