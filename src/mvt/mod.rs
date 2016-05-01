pub mod vector_tile; // protoc --rust_out . vector_tile.proto
mod geom_to_proto;

#[test]
fn test_protobuf_structs() {
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
