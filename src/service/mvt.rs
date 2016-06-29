//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{Datasource,DatasourceInput};
use datasource::PostgisInput;
use core::grid::Grid;
use core::layer::Layer;
use core::Config;
use mvt::tile::Tile;
use mvt::vector_tile;
use cache::{Cache,Tilecache,Nocache};
use toml;
use rustc_serialize::json::{Json, ToJson};


/// Collection of layers in one MVT
pub struct Tileset {
    pub name: String,
    pub layers: Vec<Layer>,
}

/// Mapbox Vector Tile Service
pub struct MvtService {
    pub input: PostgisInput,
    pub grid: Grid,
    pub tilesets: Vec<Tileset>,
    pub cache: Tilecache,
}

impl MvtService {
    fn get_tileset(&self, name: &str) -> Vec<&Layer> {
        let tileset = self.tilesets.iter().find(|t| t.name == name);
        match tileset {
            Some(set) => set.layers.iter().map(|l| l).collect(),
            None => Vec::new()
        }
    }
    fn get_tilejson_infos(&self, tileset: &str) -> (Json, Json, Json) {
        let layers = self.get_tileset(tileset);
        let metadata = Json::from_str(&format!(r#"
        {{
            "id": "{}",
            "name": "{}",
            "description": "{}",
            "attribution": "",
            "format": "pbf",
            "version": "2.0.0",
            "scheme": "xyz",
            "bounds": [-180.0,-90.0,180.0,90.0],
            "minzoom": 0,
            "maxzoom": 14,
            "center": "0.0,0.0,2",
            "basename": "{}"
        }}"#, tileset, tileset, tileset, tileset)).unwrap();
        let layers_metadata: Vec<(String,String)> = layers.iter().map(|layer| {
            let meta = layer.metadata();
            let fields = self.input.detect_columns(&layer, 0);
            let fields_json: Vec<String> = fields.iter().map(|f| format!("\"{}\": \"\"", f)).collect();
            let layers = format!(r#"{{
                "id": "{}",
                "name": "{}",
                "description": "{}",
                "srs": "{}",
                "properties": {{
                    "minzoom": {},
                    "maxzoom": {},
                    "buffer-size": {}
                }},
                "fields": {{
                    {}
                }}
                }}"#, meta.get("id").unwrap(), meta.get("name").unwrap(),  meta.get("description").unwrap(),
                meta.get("srs").unwrap(), meta.get("minzoom").unwrap(), meta.get("maxzoom").unwrap(),
                meta.get("buffer-size").unwrap(), fields_json.join(","));
            let vector_layers = format!(r#"{{
                "id": "{}",
                "description": "{}",
                "minzoom": {},
                "maxzoom": {},
                "fields": {{
                    {}
                }}
                }}"#, meta.get("id").unwrap(), meta.get("description").unwrap(),
                meta.get("minzoom").unwrap(), meta.get("maxzoom").unwrap(), fields_json.join(","));
            (layers, vector_layers)
        }).collect();
        let layers: Vec<String> = layers_metadata.iter().map(|&(ref l, _)| l.clone()).collect();
        let layers_json = Json::from_str(&format!("[{}]", layers.join(","))).unwrap();
        let vector_layers: Vec<String> = layers_metadata.iter().map(|&(_, ref l)| l.clone()).collect();
        let vector_layers_json = Json::from_str(&format!("[{}]", vector_layers.join(","))).unwrap();
        (metadata, layers_json, vector_layers_json)
    }
    pub fn get_tilejson(&self, tileset: &str) -> String {
        let (mut metadata, _layers, vector_layers) = self.get_tilejson_infos(tileset);
        let mut obj = metadata.as_object_mut().unwrap();
        let url = Json::from_str(&format!("[\"http://127.0.0.1:6767/{}/{{z}}/{{x}}/{{y}}.pbf\"]", tileset)).unwrap();
        obj.insert("tiles".to_string(), url);
        obj.insert("vector_layers".to_string(), vector_layers);
        obj.to_json().to_string()
    }
    pub fn get_metadata(&self, tileset: &str) -> String {
        let (mut metadata, layers, vector_layers) = self.get_tilejson_infos(tileset);
        let json_str = format!(r#"
        {{
          "Layer": {},
          "vector_layers": {}
        }}"#, layers.to_string(), vector_layers.to_string());
        let metadata_vector_layers = Json::from_str(&json_str).unwrap();
        let mut obj = metadata.as_object_mut().unwrap();
        obj.insert("json".to_string(), metadata_vector_layers.to_string().to_json());
        obj.to_json().to_string()
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let extent = self.grid.tile_extent_reverse_y(xtile, ytile, zoom);
        debug!("MVT tile request {:?}", extent);
        let mut tile = Tile::new(&extent, 4096, true);
        for layer in self.get_tileset(tileset) {
            let mut mvt_layer = tile.new_layer(layer);
            self.input.retrieve_features(&layer, &extent, zoom, |feat| {
                tile.add_feature(&mut mvt_layer, feat);
            });
            tile.add_layer(mvt_layer);
        }
        tile.mvt_tile
    }
    /// Fetch or create vector tile from input at x, y, z
    pub fn tile_cached(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, gzip: bool) -> Vec<u8> {
        let path = format!("{}/{}/{}/{}.pbf", tileset, zoom, xtile, ytile);

        let mut tile: Option<Vec<u8>> = None;
        self.cache.read(&path, |mut f| {
            let mut data = Vec::new();
            let _ = f.read_to_end(&mut data);
            tile = Some(data);
        });
        if tile.is_some() {
            //TODO: unzip if gzip == false
            return tile.unwrap()
        }

        let mvt_tile = self.tile(tileset, xtile, ytile, zoom);

        let mut tilegz = Vec::new();
        Tile::write_gz_to(&mut tilegz, &mvt_tile);
        let _ = self.cache.write(&path, &tilegz);

        //TODO: return unzipped if gzip == false
        tilegz
    }
}


impl Tileset {
    pub fn tilesets_from_config(config: &toml::Value) -> Result<Vec<Self>, String> {
        config.lookup("tileset")
              .ok_or("Missing configuration entry [[tileset]]".to_string())
              .and_then(|tarr| tarr.as_slice().ok_or("Array type for [[tileset]] entry expected".to_string()))
              .and_then(|tilesets| {
                  Ok(tilesets.iter().map(|tileset| Tileset::from_config(tileset).unwrap()).collect())
              })
    }
}

impl Config<Tileset> for Tileset {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        let name = config.lookup("name")
                         .ok_or("Missing configuration entry name in [[tileset]]".to_string())
                         .and_then(|val| val.as_str().ok_or("tileset.name entry is not a string".to_string()))
                         .map(|v| v.to_string());
        let layers = try!(Layer::layers_from_config(config));
        name.and_then(|n|
            Ok(Tileset{name: n, layers: layers})
        )
    }
    fn gen_config() -> String {
        let mut config = String::new();
        config.push_str(&Layer::gen_config());
        config
    }
    fn gen_runtime_config(&self) -> String {
        let mut config = String::new();
        for layer in &self.layers {
            config.push_str(&layer.gen_runtime_config());
        }
        config
    }
}

impl Config<MvtService> for MvtService {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        let pg = try!(PostgisInput::from_config(config));
        let grid = try!(Grid::from_config(config));
        let tilesets = try!(Tileset::tilesets_from_config(config));
        let cache = try!(Tilecache::from_config(config));
        Ok(MvtService {input: pg, grid: grid,
                       tilesets: tilesets, cache: cache})
    }
    fn gen_config() -> String {
        let mut config = String::new();
        config.push_str(TOML_SERVICES);
        config.push_str(&Datasource::gen_config());
        config.push_str(&Grid::gen_config());
        config.push_str(&Tileset::gen_config());
        config.push_str(&Tilecache::gen_config());
        config
    }
    fn gen_runtime_config(&self) -> String {
        let mut config = String::new();
        config.push_str(TOML_SERVICES);
        config.push_str(&self.input.gen_runtime_config());
        config.push_str(&self.grid.gen_runtime_config());
        for tileset in &self.tilesets {
            config.push_str(&tileset.gen_runtime_config());
        }
        config.push_str(&self.cache.gen_runtime_config());
        config
    }
}


const TOML_SERVICES: &'static str = r#"# t-rex configuration

[services]
mvt = true
"#;


#[test]
pub fn test_tile_query() {
    use std::io::{self,Write};
    use std::env;

    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput {connection_url: val}),
        Result::Err(_) => { write!(&mut io::stdout(), "skipped ").unwrap(); return; }
    }.unwrap();
    let grid = Grid::web_mercator();
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    layer.query_limit = Some(1);
    let tileset = Tileset{name: "points".to_string(), layers: vec![layer]};
    let service = MvtService {input: pg, grid: grid,
                              tilesets: vec![tileset], cache: Tilecache::Nocache(Nocache)};

    let mvt_tile = service.tile("points", 33, 22, 6);
    println!("{:#?}", mvt_tile);
    let expected = r#"Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some("points"),
            features: [
                Tile_Feature {
                    id: None,
                    tags: [],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        2504,
                        3390
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            keys: [],
            values: [],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell {
                value: 0
            }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell {
        value: 0
    }
}"#;
    assert_eq!(expected, &*format!("{:#?}", mvt_tile));
}

#[test]
pub fn test_metadata() {
    use core::read_config;
    use std::io::{self,Write};
    use std::env;

    if env::var("DBCONN").is_err() {
        write!(&mut io::stdout(), "skipped ").unwrap();
        return;
    }

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let metadata = service.get_metadata("points");
    println!("{}", metadata);
    let format = r#""format":"pbf""#;
    assert!(metadata.contains(format));
    let jsonmeta = r#"\"vector_layers\":"#;
    assert!(metadata.contains(jsonmeta));
}

#[test]
pub fn test_gen_config() {
    let expected = r#"# t-rex configuration

[services]
mvt = true

[datasource]
type = "postgis"
# Connection specification (https://github.com/sfackler/rust-postgres#connecting)
url = "postgresql://user:pass@host:port/database"

[grid]
# Predefined grids: web_mercator, wgs84
predefined = "web_mercator"

[[tileset]]
name = "points"

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#fid_field = "id"
#query = "SELECT name,wkb_geometry FROM mytable"

#[cache.file]
#base = "/tmp/mvtcache"
"#;
    println!("{}", &MvtService::gen_config());
    assert_eq!(expected, &MvtService::gen_config());
}
