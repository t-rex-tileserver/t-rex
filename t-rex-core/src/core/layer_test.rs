//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;
use core::config::Config;
use service::tileset::Tileset;


fn layer_from_config(toml: &str) -> Result<Layer, String> {
    use core::parse_config;

    let config = parse_config(toml.to_string(), "");
    Layer::from_config(&config?)
}

#[test]
fn test_toml_decode() {
    // Layer config with zoom level dependent queries
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        #query = "SELECT name,wkb_geometry FROM ne_10m_populated_places"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 2
        sql = "SELECT name,wkb_geometry FROM places_z2"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 10
        maxzoom = 14
        sql = "SELECT name,wkb_geometry FROM places_z10"
        "#;
    let ref cfg = layer_from_config(toml).unwrap();

    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points");
    assert_eq!(cfg.table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(cfg.query.len(), 2);
    assert_eq!(cfg.query[0].minzoom(), 2);
    assert_eq!(cfg.query[0].maxzoom(), 22);
    assert_eq!(cfg.query[1].minzoom, Some(10));
    assert_eq!(cfg.query[1].minzoom(), 10);
    assert_eq!(cfg.query[1].maxzoom(), 14);
    assert_eq!(cfg.query[1].sql,
               Some("SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.minzoom(), 2);
    assert_eq!(cfg.maxzoom(), 22);
    assert_eq!(cfg.query(1), None);
    assert_eq!(cfg.query(2),
               Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));
    assert_eq!(cfg.query(9),
               Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));
    assert_eq!(cfg.query(10),
               Some(&"SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.query(14),
               Some(&"SELECT name,wkb_geometry FROM places_z10".to_string()));
    assert_eq!(cfg.query(15),
               Some(&"SELECT name,wkb_geometry FROM places_z2".to_string()));

    // Minimal config
    let toml = r#"
        #[[tileset.layer]]
        name = "points2"
        "#;
    let cfg = layer_from_config(toml).unwrap();
    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points2");
    assert_eq!(cfg.table_name, None);
    assert_eq!(cfg.query.len(), 0);
    assert_eq!(cfg.minzoom(), 0);
    assert_eq!(cfg.maxzoom(), 22);

    // Invalid config: missing required field
    let toml = r#"
        #[[tileset.layer]]
        table_name = "missing_name"
        "#;
    let cfg = layer_from_config(toml);
    println!("{:?}", cfg);
    assert_eq!(cfg.err(), Some(" - missing field `name`".to_string()));

    // Invalid config: wrong field name
    let toml = r#"
        #[[tileset.layer]]
        name = "points3"
        tabel_name = "spelling error"
        "#;
    let cfg = layer_from_config(toml);
    println!("{:?}", cfg);

    // toml::Decoder ignores unknown keys!
    assert!(cfg.err().is_none());

    // Invalid config: wrong field type
    let toml = r#"
        #[[tileset.layer]]
        name = "points4"
        table_name = 0
        "#;
    let cfg = layer_from_config(toml);
    println!("{:?}", cfg);
    assert_eq!(cfg.err(),
               Some(" - invalid type: integer `0`, expected a string for key `table_name`"
                        .to_string()));
}

#[test]
fn test_layers_from_config() {
    use core::parse_config;
    use core::config::TilesetCfg;

    let toml = r#"
        #[[tileset]]
        name = "ne"

        #[[tileset.layer]]
        [[layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        buffer_size = 10
        [[tileset.layer.query]]
        sql = "SELECT name,wkb_geometry FROM ne_10m_populated_places"

        #[[tileset.layer]]
        [[layer]]
        name = "layer2"
        buffer-size = 10
        "#;

    let config: TilesetCfg = parse_config(toml.to_string(), "").unwrap();
    let tileset = Tileset::from_config(&config).unwrap();
    let layers = tileset.layers;
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].name, "points");
    assert_eq!(layers[0].table_name,
               Some("ne_10m_populated_places".to_string()));
    assert_eq!(layers[0].buffer_size, Some(10));
    assert_eq!(layers[1].table_name, None);
    assert_eq!(layers[1].buffer_size, None); // serde distincts between '-' and '_'

    // errors
    let emptyconfig: Result<TilesetCfg, _> = parse_config("".to_string(), "");
    assert_eq!(emptyconfig.err(),
               Some(" - missing field `name`".to_string()));
}
