//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod datasource;
pub mod postgis;
#[cfg(test)]
mod postgis_test;

pub use self::datasource::DatasourceInput;
pub use self::postgis::PostgisInput;

use core::Config;
use core::config::DatasourceCfg;


pub enum Datasource {
    Postgis(PostgisInput),
}

impl<'a> Config<'a, Datasource, DatasourceCfg> for Datasource {
    fn from_config(ds_cfg: &DatasourceCfg) -> Result<Self, String> {
        match ds_cfg.dstype.as_str() {
            "postgis" => {
                PostgisInput::from_config(ds_cfg).and_then(|pg| Ok(Datasource::Postgis(pg)))
            }
            _ => Err(format!("Unsupported datasource '{}'", ds_cfg.dstype)),
        }
    }
    fn gen_config() -> String {
        PostgisInput::gen_config()
    }
    fn gen_runtime_config(&self) -> String {
        match self {
            &Datasource::Postgis(ref pg) => pg.gen_runtime_config(),
        }
    }
}


#[cfg(test)]
fn ds_from_config(toml: &str) -> Result<Datasource, String> {
    use core::parse_config;

    let config = parse_config(toml.to_string(), "");
    Datasource::from_config(&config?)
}

#[test]
fn test_datasource_from_config() {
    let toml = r#"
        #[datasource]
        type = "postgis"
        url = "postgresql://pi@localhost/natural_earth_vectors"
        "#;
    let pg = match ds_from_config(toml).unwrap() {
        Datasource::Postgis(pg) => pg,
    };
    assert_eq!(pg.connection_url,
               "postgresql://pi@localhost/natural_earth_vectors");
}

#[test]
fn test_datasource_config_errors() {
    assert_eq!(ds_from_config("").err(),
               Some(" - missing field `type`".to_string()));

    let toml = r#"
        #[datasource]
        url = "postgresql://pi@localhost/natural_earth_vectors"
        "#;
    assert_eq!(ds_from_config(toml).err(),
               Some(" - missing field `type`".to_string()));

    let toml = r#"
        #[datasource]
        type = "postgis"
        "#;
    assert_eq!(ds_from_config(toml).err(),
               Some(" - missing field `url`".to_string()));

    let toml = r#"
        #[datasource]
        type = "postgis"
        url = true
        "#;
    assert_eq!(ds_from_config(toml).err(),
               Some(" - invalid type: boolean `true`, expected a string for key `url`"
                        .to_string()));
}
