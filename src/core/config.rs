//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml::{Value, Parser};
use std::io::prelude::*;
use std::fs::File;


pub trait Config<T> {
    /// Read configuration
    fn from_config(config: &Value) -> Result<T, String>;
    /// Generate configuration template
    fn gen_config() -> String;
    /// Generate configuration template with runtime information
    fn gen_runtime_config(&self) -> String {
        Self::gen_config()
    }
}

/// Load and parse the config file into Toml table structure.
pub fn read_config(path: &str) -> Result<Value, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_)  => {
            return Err("Could not find config file!".to_string());
        }
    };
    let mut config_toml = String::new();
    if let Err(err) = file.read_to_string(&mut config_toml) {
        return Err(format!("Error while reading config: [{}]", err));
    };

    parse_config(config_toml, path)
}

/// Parse the configuration into Toml table structure.
pub fn parse_config(config_toml: String, path: &str) -> Result<Value, String> {
    let mut parser = Parser::new(&config_toml);
    let toml = parser.parse();
    if toml.is_none() {
        let mut errors = Vec::new();
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            errors.push(format!("{}:{}:{}-{}:{} error: {}",
                     path, loline, locol, hiline, hicol, err.desc));
        }
        return Err(errors.join("\n"));
    }

    Ok(Value::Table(toml.unwrap()))
 }

#[test]
fn test_parse_config() {
    let config = read_config("src/test/example.cfg").unwrap();
    println!("{:#?}", config.as_table().unwrap());
    let expected_begin = r#"{
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
    "service": Table(
        {
            "mvt": Table(
                {
                    "viewer": Boolean(
                        true
                    )
                }
            )
        }
    ),
    "tileset": Array(
        [
            Table(
                {
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
                                    "table_name": String(
                                        "ne_10m_populated_places"
                                    )
                                }
                            ),"#;

    let expected_end = r#",
    "webserver": Table(
        {
            "bind": String(
                "0.0.0.0"
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
    assert!(format!("{:#?}", config.as_table().unwrap()).starts_with(expected_begin));
    assert!(format!("{:#?}", config.as_table().unwrap()).ends_with(expected_end));

    assert_eq!(config.lookup("datasource.type").unwrap().as_str(), Some("postgis"));
}

#[test]
fn test_parse_error() {
    let config = read_config("src/core/mod.rs");
    assert_eq!("src/core/mod.rs:0:0-0:0 error: expected a key but found an empty string", config.err().unwrap());

    let config = read_config("wrongfile");
    assert_eq!("Could not find config file!", config.err().unwrap());
}
