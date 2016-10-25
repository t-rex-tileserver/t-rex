//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;
use core::feature::{Feature,FeatureAttrValType};
use core::grid::Extent;
use core::geom::GeometryType;
use core::geom;
use core::screen;
use mvt::vector_tile;
use mvt::geom_encoder::{EncodableGeom,CommandSequence};
use protobuf::stream::CodedOutputStream;
use protobuf::core::Message;
use protobuf::error::ProtobufError;
use protobuf::parse_from_reader;
use std::fs::File;
use std::io::{BufReader,Read,Write};
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;


pub struct Tile<'a> {
    pub mvt_tile: vector_tile::Tile,
    tile_size: u32,
    extent: &'a Extent,
    reverse_y: bool,
}


impl GeometryType {
    /// GeometryType to MVT geom type
    pub fn mvt_field_type(&self) -> vector_tile::Tile_GeomType {
        match self {
            &GeometryType::Point(_)              => vector_tile::Tile_GeomType::POINT,
            &GeometryType::LineString(_)         => vector_tile::Tile_GeomType::LINESTRING,
            &GeometryType::Polygon(_)            => vector_tile::Tile_GeomType::POLYGON,
            &GeometryType::MultiPoint(_)         => vector_tile::Tile_GeomType::POINT,
            &GeometryType::MultiLineString(_)    => vector_tile::Tile_GeomType::LINESTRING,
            &GeometryType::MultiPolygon(_)       => vector_tile::Tile_GeomType::POLYGON,
            &GeometryType::GeometryCollection(_) => vector_tile::Tile_GeomType::UNKNOWN
        }
    }
}


trait ScreenGeom<T> {
    /// Convert geometry into screen coordinates
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, geom: &T) -> Self;
}

impl ScreenGeom<geom::Point> for screen::Point {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, point: &geom::Point) -> Self {
        let x_span = extent.maxx - extent.minx;
        let y_span = extent.maxy - extent.miny;
        let mut screen_geom = screen::Point {
            x: ((point.x-extent.minx) * tile_size as f64 / x_span) as i32,
            y: ((point.y-extent.miny) * tile_size as f64 / y_span) as i32
        };
        if reverse_y { screen_geom.y = tile_size as i32 - screen_geom.y };
        screen_geom
    }
}

impl ScreenGeom<geom::MultiPoint> for screen::MultiPoint {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, multipoint: &geom::MultiPoint) -> Self {
        let mut screen_geom = screen::MultiPoint { points: Vec::new() };
        for point in &multipoint.points {
            screen_geom.points.push(screen::Point::from_geom(extent, reverse_y, tile_size, point));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::LineString> for screen::LineString {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, line: &geom::LineString) -> Self {
        let mut screen_geom = screen::LineString { points: Vec::new() };
        for point in &line.points {
            screen_geom.points.push(screen::Point::from_geom(extent, reverse_y, tile_size, point));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::MultiLineString> for screen::MultiLineString {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, multiline: &geom::MultiLineString) -> Self {
        let mut screen_geom = screen::MultiLineString { lines: Vec::new() };
        for line in &multiline.lines {
            screen_geom.lines.push(screen::LineString::from_geom(extent, reverse_y, tile_size, line));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::Polygon> for screen::Polygon {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, polygon: &geom::Polygon) -> Self {
        let mut screen_geom = screen::Polygon { rings: Vec::new() };
        for line in &polygon.rings {
            screen_geom.rings.push(screen::LineString::from_geom(extent, reverse_y, tile_size, line));
        }
        screen_geom
    }
}

impl ScreenGeom<geom::MultiPolygon> for screen::MultiPolygon {
    fn from_geom(extent: &Extent, reverse_y: bool, tile_size: u32, multipolygon: &geom::MultiPolygon) -> Self {
        let mut screen_geom = screen::MultiPolygon { polygons: Vec::new() };
        for polygon in &multipolygon.polygons {
            screen_geom.polygons.push(screen::Polygon::from_geom(extent, reverse_y, tile_size, polygon));
        }
        screen_geom
    }
}


#[test]
fn test_point_to_screen_coords() {
    //let zh_mercator = geom::Point::new(949398.0, 6002729.0, Some(3857));
    let zh_mercator = geom::Point::new(960000.0, 6002729.0, Some(3857));
    //let zh_wgs84 = postgis::Point::new(47.3703149, 8.5285874, Some(4326));
    let tile_extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let screen_pt = screen::Point::from_geom(&tile_extent, false, 4096, &zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    assert_eq!(screen_pt.encode().vec(), &[9,490,6262]);
}


// --- Tile creation functions

impl<'a> Tile<'a> {
    pub fn new(extent: &Extent, tile_size: u32, reverse_y: bool) -> Tile {
        let mvt_tile = vector_tile::Tile::new();
        Tile {mvt_tile: mvt_tile, tile_size: tile_size, extent: extent, reverse_y: reverse_y }
    }

    pub fn new_layer(&mut self, layer: &Layer) -> vector_tile::Tile_Layer {
        let mut mvt_layer = vector_tile::Tile_Layer::new();
        mvt_layer.set_version(2);
        mvt_layer.set_name(layer.name.clone());
        mvt_layer.set_extent(self.tile_size);
        mvt_layer
    }

    pub fn encode_geom(&self, geom: geom::GeometryType) -> CommandSequence {
        match geom {
            GeometryType::Point(ref g) =>
                screen::Point::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::MultiPoint(ref g) =>
                screen::MultiPoint::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::LineString(ref g) =>
                screen::LineString::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::MultiLineString(ref g) =>
                screen::MultiLineString::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::Polygon(ref g) =>
                screen::Polygon::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::MultiPolygon(ref g) =>
                screen::MultiPolygon::from_geom(&self.extent, self.reverse_y, self.tile_size, g).encode(),
            GeometryType::GeometryCollection(_) => panic!("GeometryCollection not supported")
        }
    }

    fn add_feature_attribute(mvt_layer: &mut vector_tile::Tile_Layer,
                     mvt_feature: &mut vector_tile::Tile_Feature,
                     key: String, mvt_value: vector_tile::Tile_Value) {
        let keyentry = mvt_layer.get_keys().iter().position(|k| *k == key);
        // Optimization: maintain a hash table with key/index pairs
        let keyidx = match keyentry {
            None => {
                mvt_layer.mut_keys().push(key);
                mvt_layer.get_keys().len()-1
            },
            Some(idx) => idx
        };
        mvt_feature.mut_tags().push(keyidx as u32);

        let valentry = mvt_layer.get_values().iter().position(|v| *v == mvt_value);
        // Optimization: maintain a hash table with value/index pairs
        let validx = match valentry {
            None => {
                mvt_layer.mut_values().push(mvt_value);
                mvt_layer.get_values().len()-1
            },
            Some(idx) => idx
        };
        mvt_feature.mut_tags().push(validx as u32);
    }

    pub fn add_feature(&self, mut mvt_layer: &mut vector_tile::Tile_Layer, feature: &Feature) {
        let mut mvt_feature = vector_tile::Tile_Feature::new();
        if let Some(fid) = feature.fid() {
            mvt_feature.set_id(fid);
        }
        for attr in feature.attributes() {
            let mut mvt_value = vector_tile::Tile_Value::new();
            match attr.value {
                FeatureAttrValType::String(ref v) => { mvt_value.set_string_value(v.clone()); }
                FeatureAttrValType::Double(v) => { mvt_value.set_double_value(v); }
                FeatureAttrValType::Float(v) => { mvt_value.set_float_value(v); }
                FeatureAttrValType::Int(v) => { mvt_value.set_int_value(v); }
                FeatureAttrValType::UInt(v) => { mvt_value.set_uint_value(v); }
                FeatureAttrValType::SInt(v) => { mvt_value.set_sint_value(v); }
                FeatureAttrValType::Bool(v) => { mvt_value.set_bool_value(v); }
            }
            Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
                attr.key.clone(), mvt_value);
        }
        if let Ok(geom) = feature.geometry() {
            if !geom.is_empty() {
                mvt_feature.set_field_type(geom.mvt_field_type());
                mvt_feature.set_geometry(self.encode_geom(geom).vec());
                mvt_layer.mut_features().push(mvt_feature);
            }
        }
    }

    pub fn add_layer(&mut self, mvt_layer: vector_tile::Tile_Layer) {
        self.mvt_tile.mut_layers().push(mvt_layer);
    }

    pub fn write_to(mut out: &mut Write, mvt_tile: &vector_tile::Tile) {
        let mut os = CodedOutputStream::new(&mut out);
        let _ = mvt_tile.write_to(&mut os);
        os.flush().unwrap();
    }

    pub fn write_gz_to(out: &mut Write, mvt_tile: &vector_tile::Tile) {
        let mut gz = GzEncoder::new(out, Compression::Default);
        {
            let mut os = CodedOutputStream::new(&mut gz);
            let _ = mvt_tile.write_to(&mut os);
            os.flush().unwrap();
        }
        let _ = gz.finish();
    }

    pub fn read_from(fin: &mut Read) -> Result<vector_tile::Tile, ProtobufError> {
        let mut reader = BufReader::new(fin);
        parse_from_reader::<vector_tile::Tile>(&mut reader)
    }

    pub fn read_gz_from(fin: &mut Read) -> Result<vector_tile::Tile, ProtobufError> {
        let gz = GzDecoder::new(fin).unwrap();
        let mut reader = BufReader::new(gz);
        parse_from_reader::<vector_tile::Tile>(&mut reader)
    }

    pub fn binary_tile(mvt_tile: &vector_tile::Tile) -> Vec<u8> {
        let mut v = Vec::new();
        Self::write_to(&mut v, mvt_tile);
        v
    }

    pub fn to_file(&self, fname: &str) {
        let mut f = File::create(fname).unwrap();
        Self::write_to(&mut f, &self.mvt_tile);
    }
}


#[cfg(test)] use core::feature::{FeatureStruct,FeatureAttr};

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
    let mut f = File::open("src/test/tile.pbf").unwrap();
    let tile = Tile::read_from(&mut f).unwrap();
    println!("{:#?}", tile);
    let ref layer = tile.get_layers()[0];
    assert_eq!(layer.get_name(), "roads");
    let ref feature = layer.get_features()[1];
    assert_eq!(feature.get_field_type(), vector_tile::Tile_GeomType::POLYGON);
    let ref geometry = feature.get_geometry();
    assert_eq!(geometry, &[9,8236,4926,34,9,24,37,21,10,7,4,19,15]);
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
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("hello"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("world"));
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("h"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_double_value(1.23);
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("count"), mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_feature = vector_tile::Tile_Feature::new();
    mvt_feature.set_id(2);
    mvt_feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    mvt_feature.set_geometry([9, 490, 6262].to_vec());

    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_string_value(String::from("again"));
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("hello"), mvt_value);
    let mut mvt_value = vector_tile::Tile_Value::new();
    mvt_value.set_int_value(2);
    Tile::add_feature_attribute(&mut mvt_layer, &mut mvt_feature,
        String::from("count"), mvt_value);

    mvt_layer.mut_features().push(mvt_feature);

    mvt_tile.mut_layers().push(mvt_layer);
    println!("{:#?}", mvt_tile);
    assert_eq!(TILE_EXAMPLE, &*format!("{:#?}", mvt_tile));
}

#[test]
fn test_build_mvt_with_helpers() {
    use std::env;

    let extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let mut tile = Tile::new(&extent, 4096, false);
    let layer = Layer::new("points");
    let mut mvt_layer = tile.new_layer(&layer);

    let geom : GeometryType = GeometryType::Point(geom::Point::new(960000.0, 6002729.0, Some(3857)));
    let feature = FeatureStruct {
        fid: Some(1),
        attributes: vec![
            FeatureAttr {key: String::from("hello"), value: FeatureAttrValType::String(String::from("world"))},
            FeatureAttr {key: String::from("h"),     value: FeatureAttrValType::String(String::from("world"))},
            FeatureAttr {key: String::from("count"), value: FeatureAttrValType::Double(1.23)}
        ],
        geometry: geom
    };
    tile.add_feature(&mut mvt_layer, &feature);

    let geom : GeometryType = GeometryType::Point(geom::Point::new(960000.0, 6002729.0, Some(3857)));
    let feature = FeatureStruct {
        fid: Some(2),
        attributes: vec![
            FeatureAttr {key: String::from("hello"), value: FeatureAttrValType::String(String::from("again"))},
            FeatureAttr {key: String::from("count"), value: FeatureAttrValType::Int(2)}
        ],
        geometry: geom
    };
    tile.add_feature(&mut mvt_layer, &feature);

    tile.add_layer(mvt_layer);
    println!("{:#?}", tile.mvt_tile);
    assert_eq!(TILE_EXAMPLE, &*format!("{:#?}", tile.mvt_tile));

    let mut path = env::temp_dir();
    path.push("out.pbf");
    tile.to_file(&format!("{}", &path.display()));
}
