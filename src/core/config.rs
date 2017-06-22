//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml::Value;
use std::io::prelude::*;
use std::fs::File;
use core::grid::Extent;
use serde::Deserialize;


pub trait Config<'a, T, C: Deserialize<'a>> {
    /// Read configuration
    fn from_config(_config: &C) -> Result<T, String>;
    /// Generate configuration template
    fn gen_config() -> String;
    /// Generate configuration template with runtime information
    fn gen_runtime_config(&self) -> String {
        Self::gen_config()
    }
}

#[derive(Deserialize, Debug)]
pub struct ApplicationCfg {
    pub service: ServiceCfg,
    pub datasource: DatasourceCfg,
    pub grid: GridCfg,
    #[serde(rename = "tileset")]
    pub tilesets: Vec<TilesetCfg>,
    pub cache: Option<CacheCfg>,
    pub webserver: WebserverCfg,
}

#[derive(Deserialize, Debug)]
pub struct ServiceCfg {
    pub mvt: ServiceMvtCfg,
}

#[derive(Deserialize, Debug)]
pub struct ServiceMvtCfg {
    pub viewer: bool,
}

#[derive(Deserialize, Debug)]
pub struct DatasourceCfg {
    #[serde(rename = "type")]
    pub dstype: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct GridCfg {
    pub predefined: Option<String>,
    // TODO: put custom grid into [grid.user] to get rid of Option types
    /// The width and height of an individual tile, in pixels.
    pub width: Option<u16>,
    pub height: Option<u16>,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: Option<Extent>,
    /// Spatial reference system (PostGIS SRID).
    pub srid: Option<i32>,
    /// Grid units
    pub units: Option<String>,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    #[serde(default)]
    pub resolutions: Vec<f64>,
    /// Grid origin
    pub origin: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct TilesetCfg {
    pub name: String,
    pub extent: Option<Extent>,
    //? pub minzoom: Option<u8>,
    //? pub maxzoom: Option<u8>,
    //? pub center: [0.0, 0.0, 2],
    #[serde(rename = "layer")]
    pub layers: Vec<LayerCfg>,
    // Inline style
    pub style: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct LayerQueryCfg {
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub sql: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct LayerCfg {
    pub name: String,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    /// Spatial reference system (PostGIS SRID)
    pub srid: Option<i32>,
    pub fid_field: Option<String>,
    // Input for derived queries
    pub table_name: Option<String>,
    pub query_limit: Option<u32>,
    // Explicit queries
    #[serde(default)]
    pub query: Vec<LayerQueryCfg>,
    /// Simplify geometry (lines and polygons)
    pub simplify: Option<bool>,
    /// Tile buffer size in pixels
    pub buffer_size: Option<u32>,
    // Inline style
    pub style: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct CacheCfg {
    pub file: CacheFileCfg,
}

#[derive(Deserialize, Debug)]
pub struct CacheFileCfg {
    pub base: String,
    pub baseurl: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WebserverCfg {
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub threads: Option<u8>,
}

pub const DEFAULT_CONFIG: &'static str = r#"
[service.mvt]
viewer = true

[datasource]
type = "postgis"
url = ""

[grid]
predefined = "web_mercator"

[[tileset]]
name = ""

[[tileset.layer]]
name = ""

[webserver]
bind = "127.0.0.1"
port = 6767
threads = 4
"#;

/// Load and parse the config file into an config struct.
pub fn read_config<'a, T: Deserialize<'a>>(path: &str) -> Result<T, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => {
            return Err("Could not find config file!".to_string());
        }
    };
    let mut config_toml = String::new();
    if let Err(err) = file.read_to_string(&mut config_toml) {
        return Err(format!("Error while reading config: [{}]", err));
    };

    parse_config(config_toml, path)
}

/// Parse the configuration into an config struct.
pub fn parse_config<'a, T: Deserialize<'a>>(config_toml: String, path: &str) -> Result<T, String> {
    config_toml
        .parse::<Value>()
        .and_then(|cfg| cfg.try_into::<T>())
        .map_err(|err| format!("{} - {}", path, err))
}
