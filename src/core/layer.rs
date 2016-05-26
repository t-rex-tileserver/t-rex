//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use config::Config;
use toml;


#[derive(Default)]
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
    pub fn layers_from_config(config: &toml::Value) -> Result<Vec<Layer>, String> {
        config.lookup("layer")
              .ok_or("Missing configuration entry [[layer]]".to_string())
              .and_then(|larr| larr.as_slice().ok_or("Array type for [[layer]] entry expected".to_string()))
              .and_then(|layers| {
                 Ok(layers.iter().map(|layer| Layer::from_config(layer).unwrap()).collect())
               })
    }
}

impl Config<Layer> for Layer {
    fn from_config(layerval: &toml::Value) -> Result<Self, String> {
        let name = layerval.lookup("name")
                           .ok_or("Missing configuration entry name in [[layer]]".to_string())
                           .and_then(|val| val.as_str().ok_or("layer.name entry is not a string".to_string()))
                           .map(|v| v.to_string());
        let table_name =     layerval.lookup("table_name")
                                     .and_then(|val| val.as_str().map(|v| v.to_string()));
        let geometry_field = layerval.lookup("geometry_field")
                                     .and_then(|val| val.as_str().map(|v| v.to_string()));
        let geometry_type =  layerval.lookup("geometry_type")
                                     .and_then(|val| val.as_str().map(|v| v.to_string()));
        let fid_field =      layerval.lookup("fid_field")
                                     .and_then(|val| val.as_str().map(|v| v.to_string()));
        let query_limit =    layerval.lookup("query_limit")
                                     .and_then(|val| val.as_integer().map(|v| v as u32));
        let query =          layerval.lookup("query")
                                     .and_then(|val| val.as_str().map(|v| v.to_string()));
        name.and_then(|n|
            Ok(Layer { name:          n,
                   table_name:     table_name,
                   geometry_field: geometry_field,
                   geometry_type:  geometry_type,
                   fid_field:      fid_field,
                   query_limit:    query_limit,
                   query:          query,
            }))
    }

    fn gen_config() -> String {
        let toml = r#"
[[layer]]
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
        let mut lines = vec!["\n[[layer]]".to_string()];
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


#[test]
fn test_layers_from_config() {
    use config::parse_config;
    let toml = r#"
        [[layer]]
        name = "points"
        table_name = "ne_10m_populated_places"
        geometry_field = "wkb_geometry"
        geometry_type = "POINT"
        fid_field = "id"
        query_limit = 100
        query = "SELECT name,wkb_geometry FROM ne_10m_populated_places"

        [[layer]]
        name = "layer2"
        "#;

    let config = parse_config(toml.to_string(), "").unwrap();

    // read array with toml API
    let layers = config.lookup("layer").unwrap().as_slice().unwrap();
    assert_eq!(layers[0].as_table().unwrap().get("name").unwrap().as_str(), Some("points"));
    assert_eq!(layers[1].lookup("name").unwrap().as_str(), Some("layer2"));

    // read single layer
    let layer = Layer::from_config(&layers[0]).unwrap();
    assert_eq!(layer.name, "points");

    // read all layers
    let layers = Layer::layers_from_config(&config).unwrap();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].name, "points");
    assert_eq!(layers[0].table_name, Some("ne_10m_populated_places".to_string()));
    assert_eq!(layers[1].table_name, None);

    // errors
    let emptyconfig = parse_config("".to_string(), "").unwrap();
    let layers = Layer::layers_from_config(&emptyconfig);
    assert_eq!(layers.err(), Some("Missing configuration entry [[layer]]".to_string()));
}
