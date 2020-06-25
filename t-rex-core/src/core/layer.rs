//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::{self, LayerCfg};
use crate::core::Config;
use crate::service::glstyle_converter::toml_style_to_gljson;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LayerQuery {
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    pub simplify: Option<bool>,
    pub tolerance: Option<String>,
    pub sql: Option<String>,
}

#[derive(Default, Clone, Debug)]
pub struct Layer {
    pub name: String,
    pub datasource: Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    /// Spatial reference system (PostGIS SRID)
    pub srid: Option<i32>,
    /// Handle geometry like one in grid SRS
    pub no_transform: bool,
    pub fid_field: Option<String>,
    // Input for derived queries
    pub table_name: Option<String>,
    pub query_limit: Option<u32>,
    // Explicit queries
    pub query: Vec<LayerQuery>,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    /// Width and height of the tiles
    pub tile_size: u32,
    /// Simplify geometry (lines and polygons)
    pub simplify: bool,
    /// Simplification tolerance (default to !pixel_width!/2)
    pub tolerance: String,
    /// Tile buffer size in pixels (None: no clipping)
    pub buffer_size: Option<u32>,
    /// Fix invalid geometries before clipping (lines and polygons)
    pub make_valid: bool,
    /// Apply ST_Shift_Longitude to (transformed) bbox
    pub shift_longitude: bool,
    // Inline style
    pub style: Option<String>,
}

impl Layer {
    pub fn new(name: &str) -> Layer {
        Layer {
            name: String::from(name),
            tile_size: 4096,
            ..Default::default()
        }
    }
    pub fn minzoom(&self) -> u8 {
        self.minzoom
            .unwrap_or(self.query.iter().map(|q| q.minzoom).min().unwrap_or(0))
    }
    pub fn maxzoom(&self, default: u8) -> u8 {
        self.maxzoom.unwrap_or(
            self.query
                .iter()
                .map(|q| q.maxzoom.unwrap_or(default))
                .max()
                .unwrap_or(default),
        )
    }
    /// Query config for zoom level
    fn query_cfg<F>(&self, level: u8, check: F) -> Option<&LayerQuery>
    where
        F: Fn(&LayerQuery) -> bool,
    {
        let mut queries = self
            .query
            .iter()
            .map(|q| (q.minzoom, q.maxzoom.unwrap_or(22), q))
            .collect::<Vec<_>>();
        queries.sort_by_key(|ref t| t.0);
        // Start at highest zoom level and find first match
        let query = queries
            .iter()
            .rev()
            .find(|ref q| level >= q.0 && level <= q.1 && check(q.2));
        query.map(|q| q.2)
    }
    /// SQL query for zoom level
    pub fn query(&self, level: u8) -> Option<&String> {
        let query_cfg = self.query_cfg(level, |q| q.sql.is_some());
        query_cfg.and_then(|q| q.sql.as_ref().and_then(|sql| Some(sql)))
    }
    /// simplify config for zoom level
    pub fn simplify(&self, level: u8) -> bool {
        let query_cfg = self.query_cfg(level, |q| q.simplify.is_some());
        query_cfg.and_then(|q| q.simplify).unwrap_or(self.simplify)
    }
    /// tolerance config for zoom level
    pub fn tolerance(&self, level: u8) -> &String {
        let query_cfg = self.query_cfg(level, |q| q.tolerance.is_some());
        query_cfg
            .and_then(|q| q.tolerance.as_ref())
            .unwrap_or(&self.tolerance)
    }
    /// Layer properties needed e.g. for metadata.json
    pub fn metadata(&self) -> HashMap<&str, String> {
        //TODO: return Zoom-Level Array
        let mut metadata: HashMap<&str, String> = HashMap::new();
        metadata.insert("id", self.name.clone());
        metadata.insert("name", self.name.clone());
        metadata.insert("description", "".to_string());
        metadata.insert("buffer-size", self.buffer_size.unwrap_or(0).to_string());
        metadata.insert("minzoom", self.minzoom().to_string());
        metadata.insert("maxzoom", self.maxzoom(22).to_string());
        metadata.insert("srs", "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over".to_string());
        metadata
    }
}

impl<'a> Config<'a, LayerCfg> for Layer {
    fn from_config(layer_cfg: &LayerCfg) -> Result<Self, String> {
        let queries = layer_cfg
            .query
            .iter()
            .map(|lq| LayerQuery {
                minzoom: lq.minzoom,
                maxzoom: lq.maxzoom,
                simplify: lq.simplify,
                tolerance: lq.tolerance.clone(),
                sql: lq.sql.clone(),
            })
            .collect();
        let style = match layer_cfg.style {
            Some(ref style) => {
                let gljson = toml_style_to_gljson(&style);
                Some(gljson)
            }
            None => None,
        };
        Ok(Layer {
            name: layer_cfg.name.clone(),
            datasource: layer_cfg.datasource.clone(), //TODO: inherit from parents if None?
            geometry_field: layer_cfg.geometry_field.clone(),
            geometry_type: layer_cfg.geometry_type.clone(),
            srid: layer_cfg.srid,
            no_transform: layer_cfg.no_transform,
            fid_field: layer_cfg.fid_field.clone(),
            table_name: layer_cfg.table_name.clone(),
            query_limit: layer_cfg.query_limit,
            query: queries,
            minzoom: layer_cfg.minzoom,
            maxzoom: layer_cfg.maxzoom,
            tile_size: layer_cfg.tile_size,
            simplify: layer_cfg.simplify,
            tolerance: layer_cfg.tolerance.clone(),
            buffer_size: layer_cfg.buffer_size,
            make_valid: layer_cfg.make_valid,
            shift_longitude: layer_cfg.shift_longitude,
            style: style,
        })
    }

    fn gen_config() -> String {
        let toml = r#"
[[tileset]]
name = "points"
#minzoom = 0
#maxzoom = 22
#attribution = "© Contributeurs de OpenStreetMap" # Acknowledgment of ownership, authorship or copyright.
#cache_limits = {minzoom = 0, maxzoom = 22, no_cache = false}

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#simplify = true
#tolerance = "!pixel_width!/2"
#buffer_size = 10
#make_valid = true
#[[tileset.layer.query]]
#minzoom = 0
#maxzoom = 22
#sql = "SELECT name,wkb_geometry FROM mytable"
"#;
        toml.to_string()
    }

    fn gen_runtime_config(&self) -> String {
        let mut lines = vec!["[[tileset.layer]]".to_string()];
        lines.push(format!(r#"name = "{}""#, self.name));
        if let Some(ref ds) = self.datasource {
            lines.push(format!("datasource = \"{}\"", ds));
        }
        match self.table_name {
            // Remove quotes for better readability
            Some(ref table_name) => {
                lines.push(format!(r#"table_name = "{}""#, table_name.replace('"', "")))
            }
            _ => lines.push(r#"#table_name = "mytable""#.to_string()),
        }
        match self.geometry_field {
            Some(ref geometry_field) => {
                lines.push(format!("geometry_field = \"{}\"", geometry_field))
            }
            _ => lines.push("#geometry_field = \"wkb_geometry\"".to_string()),
        }
        match self.geometry_type {
            Some(ref geometry_type) => lines.push(format!("geometry_type = \"{}\"", geometry_type)),
            _ => lines.push("#geometry_type = \"POINT\"".to_string()),
        }
        match self.srid {
            Some(ref srid) => lines.push(format!("srid = {}", srid)),
            _ => lines.push("#srid = 3857".to_string()),
        }
        if self.no_transform {
            lines.push(format!("no_transform = true"));
        }
        if let Some(ref fid_field) = self.fid_field {
            lines.push(format!("fid_field = \"{}\"", fid_field));
        }
        if self.tile_size != 4096 {
            lines.push(format!(r#"tile_size = "{}""#, self.tile_size));
        }
        match self.buffer_size {
            Some(ref buffer_size) => lines.push(format!("buffer_size = {}", buffer_size)),
            _ => lines.push(format!("#buffer_size = 10")),
        }
        match self.make_valid {
            true => lines.push(format!("make_valid = true")),
            _ => lines.push(format!("#make_valid = true")),
        }
        if self.shift_longitude {
            lines.push(format!("shift_longitude = true"));
        }
        if self.geometry_type != Some("POINT".to_string()) {
            // simplify is ignored for points
            lines.push(format!("simplify = {}", self.simplify));
            if self.simplify && self.tolerance != config::DEFAULT_TOLERANCE {
                lines.push(format!("tolerance = \"{}\"", self.tolerance));
            }
        }
        match self.query_limit {
            Some(ref query_limit) => lines.push(format!("query_limit = {}", query_limit)),
            _ => lines.push("#query_limit = 1000".to_string()),
        }
        match self.query(0) {
            Some(ref query) => {
                lines.push("[[tileset.layer.query]]".to_string());
                lines.push(format!("sql = \"{}\"", query))
            }
            _ => {
                lines.push("#[[tileset.layer.query]]".to_string());
            }
        }
        lines.join("\n") + "\n"
    }
}
