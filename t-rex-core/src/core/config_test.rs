//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::read_config;
use crate::core::config::ApplicationCfg;
use crate::core::config::DEFAULT_CONFIG;

#[test]
fn test_load_config() {
    let config = read_config("../t-rex-service/src/test/example.toml");
    println!("{:#?}", config);
    let config: ApplicationCfg = config.expect("load_config returned Err");
    assert!(config.service.mvt.viewer);
    assert_eq!(config.datasource.len(), 2);
    assert_eq!(config.grid.predefined, Some("web_mercator".to_string()));
    assert_eq!(config.tilesets.len(), 1);
    assert_eq!(config.tilesets[0].name, "osm");
    assert_eq!(config.tilesets[0].layers.len(), 3);
    assert_eq!(config.tilesets[0].layers[0].name, "points");
    assert!(config.cache.is_none());
    assert_eq!(config.webserver.port, Some(8080));
}

#[test]
fn test_parse_error() {
    let config: Result<ApplicationCfg, _> = read_config("src/core/mod.rs");
    assert_eq!(
        "src/core/mod.rs - unexpected character found: `/` at line 1 column 1",
        config.err().unwrap()
    );

    let config: Result<ApplicationCfg, _> = read_config("wrongfile");
    assert_eq!("Could not find config file!", config.err().unwrap());
}

#[test]
fn test_template() {
    use crate::core::parse_config;
    use std::env;

    env::set_var("MYDBCONN", "postgresql://pi@localhost/geostat");
    env::set_var("MYPORT", "9999");
    let toml = r#"
        [service.mvt]
        viewer = true

        [[datasource]]
        dbconn = "{{ env.MYDBCONN }}"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        {# env['TSNAME'] is undefined #}
        name = "{{ env['TSNAME'] | default (value="Default-Tileset") }}"

        {% for n in [1,2,3,] %}
        [[tileset.layer]]
        name = "layer {{ n }}"
        {% endfor %}

        [webserver]
        bind = "127.0.0.1"
        port = {{env["MYPORT"]}}

        [cache]
        [cache.file]
        base = "/tmp/mvtcache"
        [cache.s3]
        endpoint = "https://s3.example.com"
        region = "westeurope"
        bucket = "bucket"
        access_key = "access-key"
        secret_key = "secret-key"
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "inline.toml.tera");
    assert_eq!(config.as_ref().err(), None);
    let config = config.unwrap();
    assert_eq!(
        config.datasource[0].dbconn,
        Some("postgresql://pi@localhost/geostat".to_string())
    );
    let cache = config.cache.unwrap();
    assert_eq!(&config.tilesets[0].name, "Default-Tileset");
    assert_eq!(config.tilesets[0].layers.len(), 3);
    assert_eq!(&config.tilesets[0].layers[0].name, "layer 1");
    assert_eq!(config.webserver.port, Some(9999));
    assert_eq!(cache.file.unwrap().base, "/tmp/mvtcache");
    assert_eq!(cache.s3.unwrap().region, "westeurope");
}

#[test]
fn test_tera_error() {
    use crate::core::parse_config;

    let toml = r#"
        {% if 1 == 1 %}
        wrong endif
        {% xendifx %}
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "inline.toml.tera");
    assert_eq!(
        config.err(),
        Some("Template error: Failed to parse \'inline.toml.tera\'".to_string())
    );

    let toml = "# {{ env.UNDEFINED }}";
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "inline.toml.tera");
    assert_eq!(
        config.err(),
        Some("Template error: Variable `env.UNDEFINED` not found in context while rendering \'inline.toml.tera\'".to_string())
    );
}

#[test]
fn test_default_config() {
    use crate::core::parse_config;
    let config: ApplicationCfg = parse_config(DEFAULT_CONFIG.to_string(), "").unwrap();
    assert_eq!(config.webserver.port, Some(6767));
}

#[test]
fn test_missing_geometry_field() {
    use crate::core::parse_config;

    let toml = r#"
        [service.mvt]
        viewer = true

        [[datasource]]
        dbconn = "postgresql://user:pass@host/database"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        name = "points"

        [[tileset.layer]]
        name = "points"
        table_name = "mytable"
        #MISSING: geometry_field = "wkb_geometry"
        geometry_type = "POINT"

        [webserver]
        bind = "127.0.0.1"
        port = 6767
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "");
    assert_eq!(None, config.err()); //TODO: we should issue an error!
}

#[test]
fn test_datasource_compatibility() {
    use crate::core::parse_config;
    // datasource spec beforce 0.8
    let toml = r#"
        [service.mvt]
        viewer = true

        [datasource]
        type = "postgis"
        url = "postgresql://pi@localhost/natural_earth_vectors"

        [grid]
        predefined = "web_mercator"

        [[tileset]]
        name = ""
        attribution = "© Contributeurs de OpenStreetMap" # Acknowledgment of ownership, authorship or copyright.

        [[tileset.layer]]
        name = ""

        [webserver]
        bind = "127.0.0.1"
        port = 6767
        threads = 4
        "#;
    let config: Result<ApplicationCfg, _> = parse_config(toml.to_string(), "");
    assert_eq!(
        " - invalid type: map, expected a sequence for key `datasource`",
        config.err().unwrap()
    );
    // let config: ApplicationCfg = config.expect("load_config returned Err");
    // assert_eq!(config.datasource[0].dbconn,
    //            Some("postgresql://pi@localhost/natural_earth_vectors".to_string()));
}
