//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{DatasourceInput, PostgisInput};
#[cfg(feature = "with-gdal")]
use gdal_ds::GdalDatasource;
#[cfg(not(feature = "with-gdal"))]
use datasource::DummyDatasource;
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use core::feature::Feature;
use core::Config;
use core::config::DatasourceCfg;
use clap::ArgMatches;


pub enum Datasource {
    Postgis(PostgisInput),
    #[cfg(feature = "with-gdal")]
    Gdal(GdalDatasource),
    #[cfg(not(feature = "with-gdal"))]
    Gdal(DummyDatasource),
}

impl DatasourceInput for Datasource {
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
    fn extent_from_wgs84(&self, extent: &Extent, dest_srid: i32) -> Option<Extent> {
        match self {
            &Datasource::Postgis(ref ds) => ds.extent_from_wgs84(extent, dest_srid),
            &Datasource::Gdal(ref ds) => ds.extent_from_wgs84(extent, dest_srid),
        }
    }
    fn layer_extent(&self, layer: &Layer) -> Option<Extent> {
        match self {
            &Datasource::Postgis(ref ds) => ds.layer_extent(layer),
            &Datasource::Gdal(ref ds) => ds.layer_extent(layer),
        }
    }
    fn prepare_queries(&mut self, layer: &Layer, grid_srid: i32) {
        match self {
            &mut Datasource::Postgis(ref mut ds) => ds.prepare_queries(layer, grid_srid),
            &mut Datasource::Gdal(ref mut ds) => ds.prepare_queries(layer, grid_srid),
        }
    }
    fn retrieve_features<F>(&self, layer: &Layer, extent: &Extent, zoom: u8, grid: &Grid, read: F)
        where F: FnMut(&Feature)
    {
        match self {
            &Datasource::Postgis(ref ds) => ds.retrieve_features(layer, extent, zoom, grid, read),
            &Datasource::Gdal(ref ds) => ds.retrieve_features(layer, extent, zoom, grid, read),
        }
    }
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
            &Datasource::Postgis(ref ds) => ds.gen_runtime_config(),
            &Datasource::Gdal(ref _ds) => unimplemented!(),
        }
    }
}


impl Datasource {
    pub fn from_args(args: &ArgMatches) -> Option<Datasource> {
        if let Some(dbconn) = args.value_of("dbconn") {
            Some(Datasource::Postgis(PostgisInput::new(dbconn)))
        } else if let Some(datasource) = args.value_of("datasource") {
            #[cfg(feature = "with-gdal")]
            let ds = Some(Datasource::Gdal(GdalDatasource::new(datasource)));
            #[cfg(not(feature = "with-gdal"))]
            let ds = {
                error!("GDAL datasource not supported in this build");
                debug!("datasource: {}", datasource);
                None
            };
            ds
        } else {
            None
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
        _ => panic!(),
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

#[cfg(feature = "with-gdal")]
mod gdal_tests {

    #[test]
    fn test_gdal_datasource_from_args() {
        use super::Datasource;
        use t_rex_core::datasource::DatasourceInput;
        use clap::{App, Arg};

        const GPKG: &str = "../t-rex-gdal/natural_earth.gpkg";
        let args = App::new("t_rex")
            .arg(Arg::with_name("datasource")
                     .long("datasource")
                     .takes_value(true))
            .get_matches_from(vec!["t_rex", "--datasource", GPKG]);
        assert_eq!(args.value_of("datasource"), Some(GPKG));
        let ds = Datasource::from_args(&args);
        if let Some(Datasource::Gdal(ref gdal_ds)) = ds {
            assert_eq!(gdal_ds.path, GPKG);
        } else {
            assert!(ds.is_some());
        }
        ds.unwrap().connected();
    }

}
