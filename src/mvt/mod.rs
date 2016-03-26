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
