mod vector_tile; // protoc --rust_out . vector_tile.proto

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
