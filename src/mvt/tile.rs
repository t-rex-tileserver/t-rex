use core::layer::Layer;
use core::feature::Feature;
use core::grid::Extent;
use core::geom::GeometryType;
use core::geom;
use core::screen;
use mvt::vector_tile;
use mvt::geom_to_proto::{EncodableGeom,CommandSequence};


pub struct Tile<'a> {
    pub mvt_tile: vector_tile::Tile,
    tile_size: u32,
    extent: &'a Extent,
    feature_id: u64,
}

// --- conversion of geometries into screen coordinates

trait ScreenGeom<T> {
    fn from_geom(extent: &Extent, tile_size: u32, geom: T) -> Self;
}

impl ScreenGeom<geom::Point> for screen::Point {
    fn from_geom(extent: &Extent, tile_size: u32, point: geom::Point) -> Self {
        let x_span = extent.maxx - extent.minx;
        let y_span = extent.maxy - extent.miny;
        screen::Point {
            x: ((point.x-extent.minx) * tile_size as f64 / x_span) as i32,
            y: ((point.y-extent.miny) * tile_size as f64 / y_span) as i32
        }
    }
}


#[test]
fn test_point_to_screen_coords() {
    //let zh_mercator = geom::Point::new(949398.0, 6002729.0);
    let zh_mercator = geom::Point::new(960000.0, 6002729.0);
    //let zh_wgs84 = postgis::Point::<WGS84>::new(47.3703149, 8.5285874);
    let tile_extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let screen_pt = screen::Point::from_geom(&tile_extent, 4096, zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    //assert_eq!(screen_pt.encode().0, &[9,490,6262]);
}


// --- Tile creation functions

impl<'a> Tile<'a> {
    pub fn new(extent: &Extent, tile_size: u32) -> Tile {
        let mvt_tile = vector_tile::Tile::new();
        Tile {mvt_tile: mvt_tile, tile_size: tile_size, extent: extent, feature_id: 0 }
    }

    pub fn new_layer(&mut self, layer: Layer) -> vector_tile::Tile_Layer {
        vector_tile::Tile_Layer::new()
    }

    pub fn new_feature(&self, feature: Feature) -> vector_tile::Tile_Feature {
        vector_tile::Tile_Feature::new()
    }

    pub fn encode_geom(&self, geom: geom::GeometryType) -> CommandSequence {
        let screen_geom = match geom {
            GeometryType::Point(p) => screen::Point::from_geom(&self.extent, self.tile_size, p),
            _ => panic!("Geometry type not implemented yet")
        };
        screen_geom.encode()
    }

    pub fn add_layer(&mut self, mvt_layer: vector_tile::Tile_Layer) {

    }
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
fn test_read_pbf_file() {
    use std::fs::File;
    use std::io::BufReader;
    use protobuf::parse_from_reader;

    let f = File::open("src/test/tile.pbf").unwrap();
    // Command line decoding:
    // protoc --decode=vector_tile.Tile src/mvt/vector_tile.proto <src/test/tile.pbf
    let mut reader = BufReader::new(f);
    let tile = parse_from_reader::<vector_tile::Tile>(&mut reader).unwrap();
    println!("{:#?}", tile);
    let ref layer = tile.get_layers()[0];
    assert_eq!(layer.get_name(), "roads");
    let ref feature = layer.get_features()[1];
    assert_eq!(feature.get_field_type(), vector_tile::Tile_GeomType::POLYGON);
    let ref geometry = feature.get_geometry();
    assert_eq!(geometry, &[9,8236,4926,34,9,24,37,21,10,7,4,19,15]);
}


struct FeatureAttributes {
    keys: Vec<String>,
    values: Vec<String>,
}

impl FeatureAttributes {
    fn new() -> FeatureAttributes {
        FeatureAttributes { keys: Vec::<String>::new(), values: Vec::<String>::new() }
    }
    fn add_attribute(&mut self, key: String, value: String) -> (u32, u32) {
        let keyentry = self.keys.iter().position(|k| *k == key);
        let keyidx = match keyentry {
            None => {
                self.keys.push(key);
                self.keys.len()-1
            },
            Some(idx) => idx
        };
        let valentry = self.values.iter().position(|v| *v == value);
        let validx = match valentry {
            None => {
                self.values.push(value);
                self.values.len()-1
            },
            Some(idx) => idx
        };
        (keyidx as u32, validx as u32)
    }
}

#[test]
fn test_create_pbf() {
    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
    let mut tile = vector_tile::Tile::new();

    let mut layer = vector_tile::Tile_Layer::new();
    layer.set_version(2);
    layer.set_name(String::from("points"));
    layer.set_extent(4096);

    let mut feature_id = 1;
    let mut feature = vector_tile::Tile_Feature::new();
    feature.set_id(feature_id);
    feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    feature.set_geometry([9, 2410, 3080].to_vec());

    let mut attrs = FeatureAttributes::new();
    let (keyidx, validx) = attrs.add_attribute(
        String::from("hello"), String::from("world"));
    feature.mut_tags().push(keyidx);
    feature.mut_tags().push(validx);
    let (keyidx, validx) = attrs.add_attribute(
        String::from("h"), String::from("world"));
    feature.mut_tags().push(keyidx);
    feature.mut_tags().push(validx);
    let (keyidx, validx) = attrs.add_attribute(
        String::from("count"), String::from("1.23")); // FIXME: double_value
    feature.mut_tags().push(keyidx);
    feature.mut_tags().push(validx);

    layer.mut_features().push(feature);

    feature_id += 1;
    feature = vector_tile::Tile_Feature::new();
    feature.set_id(feature_id);
    feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    feature.set_geometry([9, 2410, 3080].to_vec());

    let (keyidx, validx) = attrs.add_attribute(
        String::from("hello"), String::from("again"));
    feature.mut_tags().push(keyidx);
    feature.mut_tags().push(validx);
    let (keyidx, validx) = attrs.add_attribute(
        String::from("count"), String::from("2")); // FIXME: int_value
    feature.mut_tags().push(keyidx);
    feature.mut_tags().push(validx);

    layer.mut_features().push(feature);

    for key in attrs.keys.iter() {
        layer.mut_keys().push(key.clone());
    }
    for val in attrs.values.iter() {
        let mut value = vector_tile::Tile_Value::new();
        value.set_string_value(val.clone());
        layer.mut_values().push(value);
    }

    tile.mut_layers().push(layer);
    println!("{:#?}", tile);
    let expected = "Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some(\"points\"),
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
                        2410,
                        3080
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
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
                        2410,
                        3080
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                }
            ],
            keys: [
                \"hello\",
                \"h\",
                \"count\"
            ],
            values: [
                Tile_Value {
                    string_value: Some(\"world\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: Some(\"1.23\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: Some(\"again\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                },
                Tile_Value {
                    string_value: Some(\"2\"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell { value: 0 }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell { value: 0 }
}";
    assert_eq!(expected, &*format!("{:#?}", tile));
}
