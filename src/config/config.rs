//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml::{Value, Parser};
use std::io::prelude::*;
use std::fs::File;

/// Load and parse the config file into Toml table structure.
/// If a file cannot be found are cannot parsed, return None.
pub fn read_config(path: &str) -> Option<Value> {
    let mut config_toml = String::new();

    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_)  => {
            error!("Could not find config file!");
            return None;
        }
    };

    file.read_to_string(&mut config_toml)
            .unwrap_or_else(|err| panic!("Error while reading config: [{}]", err));

    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();

    if toml.is_none() {
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            println!("{}:{}:{}-{}:{} error: {}",
                     path, loline, locol, hiline, hicol, err.desc);
        }
        return None;
    }

    /*
    let config = Value::Table(toml.unwrap());
    match toml::decode(config) {
        Some(t) => t,
        None => panic!("Error while deserializing config")
    }
    */
    Some(Value::Table(toml.unwrap()))
 }

#[test]
fn test_parse_config() {
    let config = match read_config("src/test/example.cfg").unwrap() {
        Value::Table(table) => table,
        _ => panic!("Unexpected Value type")
    };
    println!("{:#?}", config);
    let expected = r#"{
    "cache": Table(
        {
            "strategy": String(
                "none"
            )
        }
    ),
    "datasource": Table(
        {
            "type": String(
                "postgis"
            ),
            "url": String(
                "postgresql://pi@localhost/natural_earth_vectors"
            )
        }
    ),
    "grid": Table(
        {
            "predefined": String(
                "web_mercator"
            )
        }
    ),
    "layer": Array(
        [
            Table(
                {
                    "fid_field": String(
                        "id"
                    ),
                    "geometry_field": String(
                        "wkb_geometry"
                    ),
                    "geometry_type": String(
                        "POINT"
                    ),
                    "name": String(
                        "points"
                    ),
                    "query": String(
                        "SELECT name,wkb_geometry FROM ne_10m_populated_places"
                    ),
                    "query_limit": Integer(
                        100
                    ),
                    "table_name": String(
                        "ne_10m_populated_places"
                    )
                }
            )
        ]
    ),
    "services": Table(
        {
            "mvt": Boolean(
                true
            )
        }
    ),
    "topics": Table(
        {
            "places": Array(
                [
                    String(
                        "points"
                    )
                ]
            )
        }
    ),
    "webserver": Table(
        {
            "bind": String(
                "0.0.0.0"
            ),
            "mapviewer": Boolean(
                true
            ),
            "port": Integer(
                8080
            ),
            "threads": Integer(
                4
            )
        }
    )
}"#;
    assert_eq!(expected, &*format!("{:#?}", config));

    for (key, value) in &config {
        println!("{}: \"{}\"", key, value);
    }

    assert!(config.contains_key("datasource"));

    let dsconfig = match config.get("datasource").unwrap() {
        &Value::Table(ref table) => table,
        _ => panic!("Unexpected Value type")
    };
    assert_eq!(format!("{}", dsconfig.get("type").unwrap()), "\"postgis\"");
}

#[test]
fn test_parse_error() {
    let config = read_config("src/config/mod.rs");
    assert_eq!(None, config);

    let config = read_config("wrongfile");
    assert_eq!(None, config);
}
