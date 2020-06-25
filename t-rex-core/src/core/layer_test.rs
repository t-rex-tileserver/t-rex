//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::Config;
use crate::core::layer::Layer;
use crate::service::tileset::Tileset;

fn layer_from_config(toml: &str) -> Result<Layer, String> {
    use crate::core::parse_config;

    let config = parse_config(toml.to_string(), "");
    Layer::from_config(&config?)
}

#[test]
fn test_query_config() {
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
    assert_eq!(cfg.query[0].minzoom, 2);
    assert_eq!(cfg.query[0].maxzoom, None);
    assert_eq!(cfg.query[1].minzoom, 10);
    assert_eq!(cfg.query[1].minzoom, 10);
    assert_eq!(cfg.query[1].maxzoom, Some(14));
    assert_eq!(
        cfg.query[1].sql,
        Some("SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(cfg.minzoom(), 2);
    assert_eq!(cfg.maxzoom(30), 30);
    assert_eq!(cfg.query(1), None);
    assert_eq!(
        cfg.query(2),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(9),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(10),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(
        cfg.query(14),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(
        cfg.query(15),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
}

#[test]
fn test_layer_defaults() {
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
    assert_eq!(cfg.maxzoom(30), 30);
}

#[test]
fn test_zoom_config() {
    // min/maxzoom in layer
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        minzoom = 1
        maxzoom = 12
        "#;
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.minzoom(), 1);
    assert_eq!(cfg.maxzoom(22), 12);

    // min/maxzoom override query limits
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        geometry_field = "wkb_geometry"
        minzoom = 1
        maxzoom = 12
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
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.minzoom(), 1);
    assert_eq!(cfg.maxzoom(22), 12);

    // handle empty query limits
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        geometry_field = "wkb_geometry"
        #[[tileset.layer.query]]
        [[query]]
        maxzoom = 13
        sql = "SELECT name,wkb_geometry FROM places_z2"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 10
        maxzoom = 14
        sql = "SELECT name,wkb_geometry FROM places_z10"
        "#;
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.minzoom(), 0);
    assert_eq!(cfg.maxzoom(22), 14);
    assert_eq!(
        cfg.query(1),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(9),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(10),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(
        cfg.query(14),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(cfg.query(15), None);

    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        geometry_field = "wkb_geometry"
        #[[tileset.layer.query]]
        [[query]]
        sql = "SELECT name,wkb_geometry FROM places_z2"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 10
        maxzoom = 14
        sql = "SELECT name,wkb_geometry FROM places_z10"
        "#;
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.minzoom(), 0);
    assert_eq!(cfg.maxzoom(22), 22);
    assert_eq!(
        cfg.query(1),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(9),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
    assert_eq!(
        cfg.query(10),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(
        cfg.query(14),
        Some(&"SELECT name,wkb_geometry FROM places_z10".to_string())
    );
    assert_eq!(
        cfg.query(15),
        Some(&"SELECT name,wkb_geometry FROM places_z2".to_string())
    );
}

#[test]
fn test_simplify_config() {
    // simplify in layer
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        simplify = true
        minzoom = 1
        maxzoom = 12
        "#;
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.simplify, true);
    assert_eq!(cfg.tolerance, "!pixel_width!/2"); // config::DEFAULT_TOLERANCE

    // simplify override ub query
    let toml = r#"
        #[[tileset.layer]]
        name = "points"
        geometry_field = "wkb_geometry"
        minzoom = 1
        maxzoom = 4
        simplify = true
        tolerance = "!pixel_width!/6"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 6
        simplify = true
        tolerance = "!pixel_width!/5"
        sql = "SELECT name,wkb_geometry FROM places_z2"
        #[[tileset.layer.query]]
        [[query]]
        minzoom = 13
        maxzoom = 13
        sql = "SELECT name,wkb_geometry FROM places_z10"
        [[query]]
        minzoom = 14
        maxzoom = 14
        simplify = false
        sql = "SELECT name,wkb_geometry FROM places_z10"
        "#;
    let cfg = layer_from_config(toml).unwrap();
    assert_eq!(cfg.simplify, true);
    assert_eq!(cfg.tolerance, "!pixel_width!/6");
    assert_eq!(cfg.simplify(1), true);
    assert_eq!(cfg.simplify(13), true);
    assert_eq!(cfg.simplify(14), false);
    assert_eq!(cfg.tolerance(1), "!pixel_width!/6");
    assert_eq!(cfg.tolerance(3), "!pixel_width!/6");
    assert_eq!(cfg.tolerance(6), "!pixel_width!/5");
    assert_eq!(cfg.tolerance(9), "!pixel_width!/5");
    assert_eq!(cfg.tolerance(13), "!pixel_width!/5");
    assert_eq!(cfg.tolerance(14), "!pixel_width!/5"); // should it be "!pixel_width!/6" ?
}

#[test]
fn test_invalid_configs() {
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
    assert_eq!(
        cfg.err(),
        Some(" - invalid type: integer `0`, expected a string for key `table_name`".to_string())
    );
}

#[test]
fn test_layers_from_config() {
    use crate::core::config::TilesetCfg;
    use crate::core::parse_config;

    let toml = r#"
        #[[tileset]]
        name = "ne"
        attribution = "© Contributeurs de OpenStreetMap" # Acknowledgment of ownership, authorship or copyright.

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
    assert_eq!(
        layers[0].table_name,
        Some("ne_10m_populated_places".to_string())
    );
    assert_eq!(layers[0].buffer_size, Some(10));
    assert_eq!(layers[1].table_name, None);
    assert_eq!(layers[1].buffer_size, None); // serde distincts between '-' and '_'

    // errors
    let emptyconfig: Result<TilesetCfg, _> = parse_config("".to_string(), "");
    assert_eq!(
        emptyconfig.err(),
        Some(" - missing field `name`".to_string())
    );
}
