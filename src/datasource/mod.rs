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
    fn from_config(config: &toml::Value) -> Option<Self> {
        config.lookup("datasource.type")
            .and_then(|val| val.as_str())
            .and_then(|tn| {
                match tn {
                    "postgis" => postgis::PostgisInput::from_config(config).and_then(|pg| Some(Datasource::Postgis(pg))),
                    _ => { error!("Unsupported datasource '{}'", tn); None }
                }
            })
    }
}


#[test]
fn test_datasource_from_config() {
    use config::config;

    let config = config::read_config("src/test/example.cfg").unwrap();
    let ds = Datasource::from_config(&config).unwrap();
    let pg = match ds { Datasource::Postgis(pg) => pg };
    assert_eq!(pg.connection_url, "postgresql://pi@localhost/natural_earth_vectors");
}
