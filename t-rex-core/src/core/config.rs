//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::gridcfg::ExtentCfg;
use regex::Regex;
use serde::Deserialize;
use std;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use tera::{Context, Tera};
use toml::Value;

pub trait Config<'a, C: Deserialize<'a>>
where
    Self: std::marker::Sized,
{
    /// Read configuration
    fn from_config(config: &C) -> Result<Self, String>;
    /// Generate configuration template
    fn gen_config() -> String;
    /// Generate configuration template with runtime information
    fn gen_runtime_config(&self) -> String {
        Self::gen_config()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ApplicationCfg {
    pub service: ServiceCfg,
    pub datasource: Vec<DatasourceCfg>,
    pub grid: GridCfg,
    #[serde(rename = "tileset")]
    pub tilesets: Vec<TilesetCfg>,
    pub cache: Option<CacheCfg>,
    pub webserver: WebserverCfg,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServiceCfg {
    pub mvt: ServiceMvtCfg,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ServiceMvtCfg {
    pub viewer: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DatasourceCfg {
    pub name: Option<String>,
    pub default: Option<bool>,
    // Postgis
    pub dbconn: Option<String>,
    pub pool: Option<u16>,
    pub connection_timeout: Option<u64>,
    // GDAL
    pub path: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GridCfg {
    pub predefined: Option<String>,
    pub user: Option<UserGridCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserGridCfg {
    /// The width and height of an individual tile, in pixels.
    pub width: u16,
    pub height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: ExtentCfg,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units (m: meters, dd: decimal degrees, ft: feet)
    pub units: String,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    #[serde(default)]
    pub resolutions: Vec<f64>,
    /// Grid origin
    pub origin: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TilesetCfg {
    pub name: String,
    pub extent: Option<ExtentCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub center: Option<(f64, f64)>,
    pub start_zoom: Option<u8>,
    pub attribution: Option<String>,
    #[serde(rename = "layer")]
    pub layers: Vec<LayerCfg>,
    // Inline style
    pub style: Option<Value>,
    pub cache_limits: Option<TilesetCacheCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LayerQueryCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    /// Simplify geometry (override layer default setting)
    pub simplify: Option<bool>,
    /// Simplification tolerance (override layer default setting)
    pub tolerance: Option<String>,
    pub sql: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct LayerCfg {
    pub name: String,
    pub datasource: Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    /// Spatial reference system (PostGIS SRID)
    pub srid: Option<i32>,
    /// Handle geometry like one in grid SRS
    #[serde(default)]
    pub no_transform: bool,
    pub fid_field: Option<String>,
    // Input for derived queries
    pub table_name: Option<String>,
    pub query_limit: Option<u32>,
    // Explicit queries
    #[serde(default)]
    pub query: Vec<LayerQueryCfg>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    /// Width and height of the tile (Default: 4096. Grid default size is 256)
    #[serde(default = "default_tile_size")]
    pub tile_size: u32,
    /// Simplify geometry (lines and polygons)
    #[serde(default)]
    pub simplify: bool,
    /// Simplification tolerance (default to !pixel_width!/2)
    #[serde(default = "default_tolerance")]
    pub tolerance: String,
    /// Tile buffer size in pixels (None: no clipping)
    pub buffer_size: Option<u32>,
    /// Fix invalid geometries before clipping (lines and polygons)
    #[serde(default)]
    pub make_valid: bool,
    /// Apply ST_Shift_Longitude to (transformed) bbox
    #[serde(default)]
    pub shift_longitude: bool,
    // Inline style
    pub style: Option<Value>,
}

pub fn default_tile_size() -> u32 {
    4096
}

pub const DEFAULT_TOLERANCE: &str = "!pixel_width!/2";

pub fn default_tolerance() -> String {
    DEFAULT_TOLERANCE.to_string()
}

#[derive(Deserialize, Clone, Debug)]
pub struct TilesetCacheCfg {
    #[serde(default)]
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    #[serde(default)]
    pub no_cache: bool,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheCfg {
    pub file: Option<CacheFileCfg>,
    pub s3: Option<S3CacheFileCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CacheFileCfg {
    pub base: String,
    pub baseurl: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct S3CacheFileCfg {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub baseurl: Option<String>,
    pub key_prefix: Option<String>,
    pub gzip_header_enabled: Option<bool>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverCfg {
    pub bind: Option<String>,
    pub port: Option<u16>,
    pub threads: Option<u8>,
    // Cache-Control headers set by web server
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Cache-Control#Expiration
    pub cache_control_max_age: Option<u32>,
    #[serde(rename = "static", default)]
    pub static_: Vec<WebserverStaticCfg>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct WebserverStaticCfg {
    pub path: String,
    pub dir: String,
}

pub const DEFAULT_CONFIG: &'static str = r#"
[service.mvt]
viewer = true

[[datasource]]
dbconn = ""

[grid]
predefined = "web_mercator"

[[tileset]]
name = ""
minzoom = 0 # Optional override of zoom limits broadcasted to tilejson descriptor
maxzoom = 22

[[tileset.layer]]
name = ""

[webserver]
bind = "127.0.0.1"
port = 6767
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
    // Check for old ${var} expressions
    let re = Regex::new(r"\$\{([[:alnum:]]+)\}").unwrap();
    if re.is_match(&config_toml) {
        return Err(
            "Replace old environment variable syntax ${VARNAME} with `{{env.VARNAME}}`".to_string(),
        );
    }

    // Parse template
    let mut tera = Tera::default();
    tera.add_raw_template(path, &config_toml)
        .map_err(|e| format!("Template error: {}", e))?;
    let mut context = Context::new();
    let mut env = HashMap::new();
    for (key, value) in env::vars() {
        env.insert(key, value);
    }
    context.insert("env", &env);
    let toml = tera
        .render(path, &context)
        .map_err(|e| format!("Template error: {}", e.source().unwrap()))?;

    toml.parse::<Value>()
        .and_then(|cfg| cfg.try_into::<T>())
        .map_err(|err| format!("{} - {}", path, err))
}
