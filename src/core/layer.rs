//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::Config;
use toml;
use rustc_serialize::Decodable;
use std::collections::HashMap;


#[derive(Default, RustcDecodable, Debug)]
pub struct Layer {
    pub name: String,
    pub table_name: Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    pub fid_field: Option<String>,
    pub query_limit: Option<u32>,
    pub query: Option<String>,
}

impl Layer {
    pub fn new(name: &str) -> Layer {
        Layer { name: String::from(name), ..Default::default() }
    }
    pub fn layers_from_config(config: &toml::Value) -> Result<Vec<Self>, String> {
        config.lookup("layer")
              .ok_or("Missing configuration entry [[tileset.layer]]".to_string())
              .and_then(|larr| larr.as_slice().ok_or("Array type for [[tileset.layer]] entry expected".to_string()))
              .and_then(|layers| {
                 Ok(layers.iter().map(|layer| Layer::from_config(layer).unwrap()).collect())
               })
    }
    /// Layer properties needed e.g. for metadata.json
    pub fn metadata(&self) -> HashMap<&str, &str> {
        //TODO: return Zoom-Level Array
        let mut metadata: HashMap<&str, &str> = HashMap::new();
        metadata.insert("id", &self.name);
        metadata.insert("name", &self.name);
        metadata.insert("description", "");
        metadata.insert("buffer-size", "0");
        metadata.insert("minzoom", "0");
        metadata.insert("maxzoom", "22");
        metadata.insert("srs", "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over");
        metadata
    }
}

impl Config<Layer> for Layer {
    fn from_config(layerval: &toml::Value) -> Result<Self, String> {
        let mut decoder = toml::Decoder::new(layerval.clone());
        let layer = Layer::decode(&mut decoder);
        layer.map_err(|e| format!("Error reading configuration - {}", e))
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
#query = "SELECT name,wkb_geometry FROM mytable"
"#;
        toml.to_string()
    }

    fn gen_runtime_config(&self) -> String {
        let mut lines = vec!["\n[[tileset]]".to_string()];
        lines.push(format!(r#"name = "{}""#, self.name));
        lines.push("\n[[tileset.layer]]".to_string());
        lines.push(format!(r#"name = "{}""#, self.name));
        match self.table_name {
            Some(ref table_name)
                => lines.push(format!(r#"table_name = "{}""#, table_name)),
            _   => lines.push(r#"#table_name = "mytable""#.to_string())
        }
        match self.geometry_field {
            Some(ref geometry_field)
                => lines.push(format!("geometry_field = \"{}\"", geometry_field)),
            _   => lines.push("#geometry_field = \"wkb_geometry\"".to_string())
        }
        match self.geometry_type {
            Some(ref geometry_type)
                => lines.push(format!("geometry_type = \"{}\"", geometry_type)),
            _   => lines.push("#geometry_type = \"POINT\"".to_string())
        }
        match self.fid_field {
            Some(ref fid_field)
                => lines.push(format!("fid_field = \"{}\"", fid_field)),
            _   => lines.push("#fid_field = \"id\"".to_string())
        }
        match self.query_limit {
            Some(ref query_limit)
                => lines.push(format!("query_limit = {}", query_limit)),
            _   => {}
        }
        match self.query {
            Some(ref query)
                => lines.push(format!("query = \"{}\"", query)),
            _   => {
                let default_name = "mytable".to_string();
                let ref table_name = self.table_name.as_ref().unwrap_or(&default_name);
                let default_name = "wkb_geometry".to_string();
                let ref geometry_field = self.geometry_field.as_ref().unwrap_or(&default_name);
                lines.push(format!("#query = \"SELECT name,{} FROM {}\"", geometry_field, table_name))
            }
        }
        lines.join("\n") + "\n"
    }
}


#[cfg(test)]
#[derive(RustcDecodable, Debug)]
struct Layerlevel {
    pub minzoom: u8,
    pub maxzoom: u8,
    pub sql: String,
}

#[cfg(test)]
#[derive(RustcDecodable, Debug)]
struct LayerConfig {
    pub name: String,
    pub table_name: Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    pub fid_field: Option<String>,
    pub query_limit: Option<u32>,
    pub query: Option<String>,
    pub level: Vec<Layerlevel>,
}

#[test]
fn test_toml_decode() {
    use core::parse_config;
    let toml = r#"
        [[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        query = "SELECT name,wkb_geometry FROM ne_10m_populated_places"
        [[tileset.layer.level]]
        minzoom = 10
        maxzoom = 14
        sql = "SELECT name,wkb_geometry FROM places_z10"

        [[tileset.layer]]
        name = "points2"

        [[tileset.layer]]
        table_name = "missing_name"

        [[tileset.layer]]
        name = "points3"
        tabel_name = "spelling error"

        [[tileset.layer]]
        name = "points4"
        table_name = 0
        "#;

    let tomlcfg = parse_config(toml.to_string(), "").unwrap();
    let layers = tomlcfg.lookup("tileset.layer").unwrap().as_slice().unwrap();

    // Layer config with zoom level dependent queries
    let ref layer = layers[0];
    let mut decoder = toml::Decoder::new(layer.clone());
    let cfg = LayerConfig::decode(&mut decoder).unwrap();
    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points");
    assert_eq!(cfg.table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(cfg.level.len(), 1);
    assert_eq!(cfg.level[0].minzoom, 10);
    assert_eq!(cfg.level[0].sql, "SELECT name,wkb_geometry FROM places_z10");

    // Minimal config
    let ref layer = layers[1];
    let mut decoder = toml::Decoder::new(layer.clone());
    let cfg = LayerConfig::decode(&mut decoder).unwrap();
    println!("{:?}", cfg);
    assert_eq!(cfg.name, "points2");
    assert_eq!(cfg.table_name, None);
    assert_eq!(cfg.level.len(), 0);

    // Invalid config: missing required field
    let ref layer = layers[2];
    let mut decoder = toml::Decoder::new(layer.clone());
    let cfg = LayerConfig::decode(&mut decoder);
    println!("{:?}", cfg);
    assert_eq!(format!("{}", cfg.err().unwrap()),
        "expected a value of type `string` for the key `name`");

    // Invalid config: wrong field name
    let ref layer = layers[3];
    let mut decoder = toml::Decoder::new(layer.clone());
    let cfg = LayerConfig::decode(&mut decoder);
    println!("{:?}", cfg);
    // toml::Decoder ignores unknown keys!
    assert_eq!(cfg.err(), None);

    // Invalid config: wrong field type
    let ref layer = layers[4];
    let mut decoder = toml::Decoder::new(layer.clone());
    let cfg = LayerConfig::decode(&mut decoder);
    println!("{:?}", cfg);
    assert_eq!(format!("{}", cfg.err().unwrap()),
        "expected a value of type `string`, but found a value of type `integer` for the key `table_name`");
}

#[test]
fn test_layers_from_config() {
    use core::parse_config;
    let toml = r#"
        [[tileset]]
        name = "ne"

        [[tileset.layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        query = "SELECT name,wkb_geometry FROM ne_10m_populated_places"

        [[tileset.layer]]
        name = "layer2"
        "#;

    let config = parse_config(toml.to_string(), "").unwrap();

    let tilesets = config.lookup("tileset").unwrap().as_slice().unwrap();
    let layers = Layer::layers_from_config(&tilesets[0]).unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].name, "points");
    assert_eq!(layers[0].table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(layers[1].table_name, None);

    // errors
    let emptyconfig = parse_config("".to_string(), "").unwrap();
    let layers = Layer::layers_from_config(&emptyconfig);
    assert_eq!(layers.err(), Some("Missing configuration entry [[tileset.layer]]".to_string()));
}
