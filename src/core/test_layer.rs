//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;


#[test]
fn test_toml_decode() {
    use core::parse_config;
    let toml = r#"
        [[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        #query = "SELECT name,wkb_geometry FROM ne_10m_populated_places"
        [[tileset.layer.query]]
        minzoom = 2
        sql = "SELECT name,wkb_geometry FROM places_z2"
        [[tileset.layer.query]]
        minzoom = 10
        maxzoom = 14
        sql = "SELECT name,wkb_geometry FROM places_z10"

        [[tileset.layer]]
        name = "points2"

        [[tileset.layer]]
        table_name = "missing_name"

        [[tileset.layer]]
        name = "points3"
        tabel_name = "spelling error"

        [[tileset.layer]]
        name = "points4"
        table_name = 0
        "#;

    let tomlcfg = parse_config(toml.to_string(), "").unwrap();
    let layers = tomlcfg["tileset.layer"].as_array().unwrap();

    // Layer config with zoom level dependent queries
    let ref layer = layers[0];
    let cfg: Layer = layer.clone().try_into().unwrap();

    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points");
    assert_eq!(cfg.table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(cfg.query.len(), 2);
    assert_eq!(cfg.query[0].minzoom(), 2);
    assert_eq!(cfg.query[0].maxzoom(), 99);
    assert_eq!(cfg.query[1].minzoom, Some(10));
    assert_eq!(cfg.query[1].minzoom(), 10);
    assert_eq!(cfg.query[1].maxzoom(), 14);
    assert_eq!(cfg.query[1].sql, Some("SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.minzoom(), 2);
    assert_eq!(cfg.maxzoom(), 99);
    assert_eq!(cfg.query(1), None);
    assert_eq!(cfg.query(2), Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));
    assert_eq!(cfg.query(9), Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));
    assert_eq!(cfg.query(10), Some(&"SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.query(14), Some(&"SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.query(15), Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));

    // Minimal config
    let ref layer = layers[1];
    let cfg: Layer = layer.clone().try_into().unwrap();

    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points2");
    assert_eq!(cfg.table_name, None);
    assert_eq!(cfg.query.len(), 0);
    assert_eq!(cfg.minzoom(), 0);
    assert_eq!(cfg.maxzoom(), 99);

    // Invalid config: missing required field
    let ref layer = layers[2];
    let cfg = layer.clone().try_into::<Layer>();
    println!("{:?}", cfg);
    assert_eq!(format!("{}", cfg.err().unwrap()),
        "expected a value of type `string` for the key `name`");

    // Invalid config: wrong field name
    let ref layer = layers[3];
    let cfg = layer.clone().try_into::<Layer>();
    println!("{:?}", cfg);
    // toml::Decoder ignores unknown keys!
    assert!(cfg.err().is_none());

    // Invalid config: wrong field type
    let ref layer = layers[4];
    let cfg = layer.clone().try_into::<Layer>();
    println!("{:?}", cfg);
    assert_eq!(format!("{}", cfg.err().unwrap()),
        "expected a value of type `string`, but found a value of type `integer` for the key `table_name`");
}

#[test]
fn test_layers_from_config() {
    use core::parse_config;
    let toml = r#"
        [[tileset]]
        name = "ne"

        [[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        buffer-size = 10
        [[tileset.layer.query]]
        sql = "SELECT name,wkb_geometry FROM ne_10m_populated_places"

        [[tileset.layer]]
        name = "layer2"
        "#;

    let config = parse_config(toml.to_string(), "").unwrap();

    let tilesets = config["tileset"].as_array().unwrap();
    let layers = Layer::layers_from_config(&tilesets[0]).unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].name, "points");
    assert_eq!(layers[0].table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(layers[0].buffer_size, Some(10));
    assert_eq!(layers[1].table_name, None);

    // errors
    let emptyconfig = parse_config("".to_string(), "").unwrap();
    let layers = Layer::layers_from_config(&emptyconfig);
    assert_eq!(layers.err(), Some("Missing configuration entry [[tileset.layer]]".to_string()));
}
