//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::Config;
use core::config::LayerCfg;
use service::glstyle_converter::toml_style_to_gljson;
use std::collections::HashMap;
use datasource::PostgisInput;


#[derive(Debug)]
pub struct LayerQuery {
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub sql: Option<String>,
}

#[derive(Default, Debug)]
pub struct Layer {
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
    pub query: Vec<LayerQuery>,
    /// Simplify geometry (lines and polygons)
    pub simplify: Option<bool>,
    /// Tile buffer size in pixels
    pub buffer_size: Option<u32>,
    // Inline style
    pub style: Option<String>,
}

impl LayerQuery {
    pub fn minzoom(&self) -> u8 {
        self.minzoom.unwrap_or(0)
    }
    pub fn maxzoom(&self) -> u8 {
        self.maxzoom.unwrap_or(22)
    }
}

impl Layer {
    pub fn new(name: &str) -> Layer {
        Layer {
            name: String::from(name),
            ..Default::default()
        }
    }
    pub fn minzoom(&self) -> u8 {
        self.query
            .iter()
            .map(|q| q.minzoom())
            .min()
            .unwrap_or(0)
    }
    pub fn maxzoom(&self) -> u8 {
        self.query
            .iter()
            .map(|q| q.maxzoom())
            .max()
            .unwrap_or(22)
    }
    // SQL query for zoom level
    pub fn query(&self, level: u8) -> Option<&String> {
        let mut queries = self.query
            .iter()
            .map(|ref q| (q.minzoom(), q.maxzoom(), q.sql.as_ref().and_then(|sql| Some(sql))))
            .collect::<Vec<_>>();
        queries.sort_by_key(|ref t| t.0);
        let query = queries
            .iter()
            .rev()
            .find(|ref q| level >= q.0 && level <= q.1);
        query.and_then(|ref q| q.2)
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
        metadata.insert("maxzoom", self.maxzoom().to_string());
        metadata.insert("srs", "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over".to_string());
        metadata
    }
    pub fn gen_runtime_config_from_input(&self, input: &PostgisInput) -> String {
        let extent = input.layer_extent(self);
        let mut lines = vec!["\n[[tileset]]".to_string()];
        lines.push(format!(r#"name = "{}""#, self.name));
        if let Some(ext) = extent {
            lines.push(format!(r#"extent = [{:.5}, {:.5}, {:.5}, {:.5}]"#,
                               ext.minx,
                               ext.miny,
                               ext.maxx,
                               ext.maxy));
        } else {
            lines.push("#extent = [-180.0,-90.0,180.0,90.0]".to_string());
        }

        let mut cfg = lines.join("\n") + "\n";
        cfg.push_str(&self.gen_runtime_config());
        if self.query(0).is_none() {
            let query = input.build_query_sql(self, 3857, None, true).unwrap();
            cfg.push_str(&format!("#sql = \"\"\"{}\"\"\"\n", query))
        }
        cfg
    }
}

impl<'a> Config<'a, Layer, LayerCfg> for Layer {
    fn from_config(layer_cfg: &LayerCfg) -> Result<Self, String> {
        let queries = layer_cfg
            .query
            .iter()
            .map(|lq| {
                     LayerQuery {
                         minzoom: lq.minzoom,
                         maxzoom: lq.maxzoom,
                         sql: lq.sql.clone(),
                     }
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
               geometry_field: layer_cfg.geometry_field.clone(),
               geometry_type: layer_cfg.geometry_type.clone(),
               srid: layer_cfg.srid,
               fid_field: layer_cfg.fid_field.clone(),
               table_name: layer_cfg.table_name.clone(),
               query_limit: layer_cfg.query_limit,
               query: queries,
               simplify: layer_cfg.simplify,
               buffer_size: layer_cfg.buffer_size,
               style: style,
           })
    }

    fn gen_config() -> String {
        let toml = r#"
[[tileset]]
name = "points"

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#fid_field = "id"
#simplify = true
#buffer_size = 10
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
        match self.table_name {
            Some(ref table_name) => lines.push(format!(r#"table_name = "{}""#, table_name)),
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
        match self.fid_field {
            Some(ref fid_field) => lines.push(format!("fid_field = \"{}\"", fid_field)),
            _ => lines.push("#fid_field = \"id\"".to_string()),
        }
        match self.buffer_size {
            Some(ref buffer_size) => lines.push(format!("buffer_size = {}", buffer_size)),
            _ => lines.push(format!("#buffer_size = 10")),
        }
        if self.geometry_type != Some("POINT".to_string()) {
            // simplify is ignored for points
            match self.simplify {
                Some(ref simplify) => lines.push(format!("simplify = {}", simplify)),
                _ => lines.push(format!("#simplify = true")),
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
