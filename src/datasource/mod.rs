//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod datasource;
pub mod postgis;

use config::Config;
use toml;


pub enum Datasource {
    Postgis(postgis::PostgisInput),
}

impl Config<Datasource> for Datasource {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        config.lookup("datasource.type")
            .ok_or("Missing configuration entry 'datasource.type'".to_string())
            .and_then(|val| val.as_str().ok_or("url entry is not a string".to_string()))
            .and_then(|tn| {
                match tn {
                    "postgis" => postgis::PostgisInput::from_config(config).and_then(|pg| Ok(Datasource::Postgis(pg))),
                    _ => { Err(format!("Unsupported datasource '{}'", tn)) }
                }
            })
    }
}


#[cfg(test)]
fn ds_from_config(toml: &str) -> Result<Datasource, String> {
    use config::parse_config;

    let config = parse_config(toml.to_string(), "").unwrap();
    Datasource::from_config(&config)
}

#[test]
fn test_datasource_from_config() {
    let toml = r#"
        [datasource]
        type = "postgis"
        url = "postgresql://pi@localhost/natural_earth_vectors"
        "#;
    let pg = match ds_from_config(toml).unwrap() { Datasource::Postgis(pg) => pg };
    assert_eq!(pg.connection_url, "postgresql://pi@localhost/natural_earth_vectors");
}

#[test]
fn test_datasource_config_errors() {
    assert_eq!(ds_from_config("").err().unwrap(), "Missing configuration entry \'datasource.type\'" );
    let toml = r#"
        [datasource]
        url = "postgresql://pi@localhost/natural_earth_vectors"
        "#;
    assert_eq!(ds_from_config(toml).err().unwrap(), "Missing configuration entry \'datasource.type\'" );
    let toml = r#"
        [datasource]
        type = "postgis"
        "#;
    assert_eq!(ds_from_config(toml).err().unwrap(), "Missing configuration entry \'datasource.url\'" );
    let toml = r#"
        [datasource]
        type = "postgis"
        url = true
        "#;
    assert_eq!(ds_from_config(toml).err().unwrap(), "url entry is not a string" );
}
