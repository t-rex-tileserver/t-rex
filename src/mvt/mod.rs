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


struct FeatureAttribute {
    key: String,
    values: Vec<String>
}

struct FeatureAttributes {
    vec: Vec<FeatureAttribute>
}

impl FeatureAttributes {
    fn new() -> FeatureAttributes {
        FeatureAttributes { vec: Vec::<FeatureAttribute>::new() }
    }
    fn add_attribute(&mut self, key: String, value: String) -> (u32, u32) {
        let entry = self.vec.iter().position(|ref kv| kv.key == key);
        match entry {
            None => {
                self.vec.push(FeatureAttribute {
                        key: key,
                        values : [value].to_vec()
                    });
                (self.vec.len() as u32 - 1, 0)
            },
            Some(idx) => {
                let mut kv = self.vec.get_mut(idx).unwrap();
                let valentry = kv.values.iter().position(|v| *v == value);
                let validx = match valentry {
                    None => {
                        kv.values.push(value);
                        kv.values.len()-1
                    },
                    Some(idx) => idx
                };
                (idx as u32, validx as u32)
            }
        }
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

    layer.mut_features().push(feature);

    feature_id += 1;
    feature = vector_tile::Tile_Feature::new();
    feature.set_id(feature_id);
    feature.set_field_type(vector_tile::Tile_GeomType::POINT);
    feature.set_geometry([9, 2410, 3080].to_vec());
    layer.mut_features().push(feature);

    for keyval in attrs.vec.iter() {
        layer.mut_keys().push(keyval.key.clone());
        for val in keyval.values.iter() {
            let mut value = vector_tile::Tile_Value::new();
            value.set_string_value(val.clone());
            layer.mut_values().push(value);
        }
    }

    tile.mut_layers().push(layer);
    println!("{:#?}", tile);
}