//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use clap::ArgMatches;
use std::collections::HashMap;
use t_rex_core::core::config::{ApplicationCfg, DatasourceCfg};
use t_rex_core::core::feature::Feature;
use t_rex_core::core::layer::Layer;
use t_rex_core::core::Config;
#[cfg(not(feature = "with-gdal"))]
use t_rex_core::datasource::DummyDatasource as GdalDatasource;
use t_rex_core::datasource::{DatasourceType, PostgisDatasource};
#[cfg(feature = "with-gdal")]
use t_rex_gdal::GdalDatasource;
use tile_grid::{Extent, Grid};

#[derive(Clone)]
pub enum Datasource {
    Postgis(PostgisDatasource),
    Gdal(GdalDatasource),
}

impl DatasourceType for Datasource {
    fn connected(&self) -> Datasource {
        match self {
            &Datasource::Postgis(ref ds) => Datasource::Postgis(ds.connected()),
            &Datasource::Gdal(ref ds) => Datasource::Gdal(ds.connected()),
        }
    }
    fn detect_layers(&self, detect_geometry_types: bool) -> Vec<Layer> {
        match self {
            &Datasource::Postgis(ref ds) => ds.detect_layers(detect_geometry_types),
            &Datasource::Gdal(ref ds) => ds.detect_layers(detect_geometry_types),
        }
    }
    fn detect_data_columns(&self, layer: &Layer, sql: Option<&String>) -> Vec<(String, String)> {
        match self {
            &Datasource::Postgis(ref ds) => ds.detect_data_columns(layer, sql),
            &Datasource::Gdal(ref ds) => ds.detect_data_columns(layer, sql),
        }
    }
    fn reproject_extent(
        &self,
        extent: &Extent,
        dest_srid: i32,
        src_srid: Option<i32>,
    ) -> Option<Extent> {
        match self {
            &Datasource::Postgis(ref ds) => ds.reproject_extent(extent, dest_srid, src_srid),
            &Datasource::Gdal(ref ds) => ds.reproject_extent(extent, dest_srid, src_srid),
        }
    }
    fn layer_extent(&self, layer: &Layer, grid_srid: i32) -> Option<Extent> {
        match self {
            &Datasource::Postgis(ref ds) => ds.layer_extent(layer, grid_srid),
            &Datasource::Gdal(ref ds) => ds.layer_extent(layer, grid_srid),
        }
    }
    fn prepare_queries(&mut self, tileset: &str, layer: &Layer, grid_srid: i32) {
        match self {
            &mut Datasource::Postgis(ref mut ds) => ds.prepare_queries(tileset, layer, grid_srid),
            &mut Datasource::Gdal(ref mut ds) => ds.prepare_queries(tileset, layer, grid_srid),
        }
    }
    fn retrieve_features<F>(
        &self,
        tileset: &str,
        layer: &Layer,
        extent: &Extent,
        zoom: u8,
        grid: &Grid,
        read: F,
    ) -> u64
    where
        F: FnMut(&dyn Feature),
    {
        match self {
            &Datasource::Postgis(ref ds) => {
                ds.retrieve_features(tileset, layer, extent, zoom, grid, read)
            }
            &Datasource::Gdal(ref ds) => {
                ds.retrieve_features(tileset, layer, extent, zoom, grid, read)
            }
        }
    }
}

impl<'a> Config<'a, DatasourceCfg> for Datasource {
    fn from_config(ds_cfg: &DatasourceCfg) -> Result<Self, String> {
        if ds_cfg.dbconn.is_some() {
            PostgisDatasource::from_config(ds_cfg).and_then(|ds| Ok(Datasource::Postgis(ds)))
        } else if ds_cfg.path.is_some() {
            GdalDatasource::from_config(ds_cfg).and_then(|ds| Ok(Datasource::Gdal(ds)))
        } else {
            Err(format!("Unsupported datasource"))
        }
    }
    fn gen_config() -> String {
        format!(
            "{}{}",
            PostgisDatasource::gen_config(),
            GdalDatasource::gen_config()
        )
    }
    fn gen_runtime_config(&self) -> String {
        match self {
            &Datasource::Postgis(ref ds) => ds.gen_runtime_config(),
            &Datasource::Gdal(ref ds) => ds.gen_runtime_config(),
        }
    }
}

#[derive(Clone)]
pub struct Datasources {
    pub datasources: HashMap<String, Datasource>,
    pub default: Option<String>,
}

impl<'a> Config<'a, ApplicationCfg> for Datasources {
    fn from_config(app_cfg: &ApplicationCfg) -> Result<Self, String> {
        let mut datasources = Datasources::new();
        let default_name = "<noname>".to_string();
        for ds_cfg in &app_cfg.datasource {
            let name = ds_cfg.name.as_ref().unwrap_or(&default_name);
            let ds = Datasource::from_config(&ds_cfg).unwrap();
            datasources.add(name, ds);
            if ds_cfg.default.unwrap_or(false) {
                datasources.default = Some(name.clone());
            }
        }
        datasources.setup();
        Ok(datasources)
    }
    fn gen_config() -> String {
        Datasource::gen_config()
    }
    fn gen_runtime_config(&self) -> String {
        let mut config = String::new();
        for (name, ds) in &self.datasources {
            config.push_str(&ds.gen_runtime_config());
            if name != "" {
                config.push_str(&format!("name = \"{}\"\n", name));
            }
            if self.default.is_some() && name == self.default.as_ref().unwrap() {
                config.push_str("default = true\n");
            }
        }
        config
    }
}

impl Datasources {
    pub fn new() -> Self {
        Datasources {
            datasources: HashMap::new(),
            default: None,
        }
    }
    pub fn add(&mut self, name: &String, ds: Datasource) {
        // TODO: check for duplicate names
        self.datasources.insert(name.clone(), ds);
    }
    pub fn from_args(args: &ArgMatches) -> Self {
        let mut datasources = Datasources::new();
        if let Some(dbconn) = args.value_of("dbconn") {
            datasources.add(
                &"dbconn".to_string(),
                Datasource::Postgis(PostgisDatasource::new(dbconn, None, None)),
            );
        }
        if let Some(datasource) = args.value_of("datasource") {
            #[cfg(feature = "with-gdal")]
            let ds = Some(Datasource::Gdal(GdalDatasource::new(datasource)));
            #[cfg(not(feature = "with-gdal"))]
            let ds = {
                error!("GDAL datasource not supported in this build");
                debug!("datasource: {}", datasource);
                None
            };
            if let Some(ds) = ds {
                datasources.add(&"datasource".to_string(), ds);
            }
        }
        datasources.setup();
        datasources
    }
    /// Finish initialization
    pub fn setup(&mut self) {
        // TODO: default should be first in config, not first in HashMap
        if self.default.is_none() {
            self.default = self.datasources.keys().cloned().next();
        }
    }
    pub fn datasource(&self, name: &Option<String>) -> Option<&Datasource> {
        let key = name.as_ref().unwrap_or(self.default.as_ref().unwrap());
        self.datasources.get(key)
    }
    pub fn datasource_mut(&mut self, name: &Option<String>) -> Option<&mut Datasource> {
        let key = name.as_ref().unwrap_or(self.default.as_ref().unwrap());
        self.datasources.get_mut(key)
    }
    pub fn default(&self) -> Option<&Datasource> {
        match self.default {
            Some(ref default) => self.datasources.get(default),
            None => None,
        }
    }
}

#[cfg(test)]
fn ds_from_config(toml: &str) -> Result<Datasource, String> {
    use t_rex_core::core::parse_config;

    let config = parse_config(toml.to_string(), "");
    Datasource::from_config(&config?)
}

#[test]
fn test_datasource_from_config() {
    let toml = r#"
        #[[datasource]]
        dbconn = "postgresql://pi@localhost/natural_earth_vectors"
        "#;
    let pg = match ds_from_config(toml).unwrap() {
        Datasource::Postgis(pg) => pg,
        _ => panic!(),
    };
    assert_eq!(
        pg.connection_url,
        "postgresql://pi@localhost/natural_earth_vectors"
    );
}

#[test]
fn test_datasource_config_errors() {
    assert_eq!(
        ds_from_config("").err(),
        Some("Unsupported datasource".to_string())
    );

    let toml = r#"
        #[[datasource]]
        pool = 10
        "#;
    assert_eq!(
        ds_from_config(toml).err(),
        Some("Unsupported datasource".to_string())
    );

    let toml = r#"
        #[[datasource]]
        dbconn = true
        "#;
    assert_eq!(
        ds_from_config(toml).err(),
        Some(" - invalid type: boolean `true`, expected a string for key `dbconn`".to_string())
    );
}

#[cfg(feature = "with-gdal")]
mod gdal_tests {

    #[test]
    fn test_gdal_datasource_from_args() {
        use super::*;
        use clap::{App, Arg};
        use t_rex_core::datasource::DatasourceType;

        const GPKG: &str = "../t-rex-gdal/natural_earth.gpkg";
        let args = App::new("t_rex")
            .arg(
                Arg::with_name("datasource")
                    .long("datasource")
                    .takes_value(true),
            )
            .get_matches_from(vec!["t_rex", "--datasource", GPKG]);
        assert_eq!(args.value_of("datasource"), Some(GPKG));
        let dss = Datasources::from_args(&args);
        if let Some(&Datasource::Gdal(ref gdal_ds)) = dss.default() {
            assert_eq!(gdal_ds.path, GPKG);
        } else {
            assert!(dss.default().is_some());
        }
        dss.default().unwrap().connected();
    }
}
