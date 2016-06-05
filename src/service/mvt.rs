//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{Datasource,DatasourceInput};
use datasource::PostgisInput;
use core::grid::{Extent,Grid};
use core::layer::Layer;
use core::Config;
use mvt::tile::Tile;
use mvt::vector_tile;
use mvt::geom_encoder::EncodableGeom;
use cache::{Cache,Tilecache,Nocache,Filecache};
use toml;
use rustc_serialize::json::{self, Json, ToJson};


/// Collection of layers in one MVT
pub struct Tileset {
    pub name: String,
    pub layers: Vec<String>,
}

/// Mapbox Vector Tile Service
pub struct MvtService {
    pub input: PostgisInput,
    pub grid: Grid,
    pub layers: Vec<Layer>,
    pub tilesets: Vec<Tileset>,
    pub cache: Tilecache,
}

impl MvtService {
    fn get_tileset(&self, name: &str) -> Vec<&Layer> {
        let tileset = self.tilesets.iter().find(|t| t.name == name);
        match tileset {
            Some(set) => set.layers.iter().map(|name| self.get_layer(name).unwrap()).collect(),
            None => vec![self.get_layer(name).unwrap()]
        }
    }
    fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|t| t.name == name)
    }
    pub fn get_metadata(&self, tileset: &str) -> String {
        let mut metadata = Json::from_str(r#"
        {
            "attribution": "",
            "description": "T-Rex cache",
            "format": "pbf",
            "filesize": "12345",
            "basename": "t_rex.mbtiles",
            "minzoom": "0",
            "name": "t_rex",
            "version": "2.0",
            "bounds": "-180.0,-90.0,180.0,90.0",
            "mtime": "1463000297761",
            "maxzoom": "14",
            "scheme": "tms",
            "type": "baselayer",
            "id": "t_rex",
            "center": "0.0,0.0,10"
        }"#).unwrap();
        let metadata_vector_layers = Json::from_str(r#"
        {
          "Layer": [
            {
              "id": "ne_10m_populated_places",
              "description": "",
              "fields": {
                "name": ""
              },
              "properties": {
                "buffer-size": 256,
                "minzoom": 0,
                "maxzoom": 22
              },
              "srs": "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over",
              "name": "ne_10m_populated_places"
            }
          ],
          "vector_layers": [
            {
              "id": "ne_10m_populated_places",
              "description": "",
              "minzoom": 0,
              "maxzoom": 22,
              "fields": {
                "name": ""
              }
            }
          ]
        }"#).unwrap();
        let mut obj = metadata.as_object_mut().unwrap();
        obj.insert("json".to_string(), metadata_vector_layers.to_string().to_json());
        obj.to_json().to_string()
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let mut tile: Option<vector_tile::Tile> = None;
        self.cache.lookup(tileset, xtile, ytile, zoom, |mut f| {
            tile = Tile::read_gz_from(&mut f).ok();
            Ok(()) //result.map(|_| ()).map_err(|e| io::Error::new(io::ErrorKind::Other, e.description()))
        });
        if tile.is_some() {
            return tile.unwrap()
        }
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
        // Write into cache
        self.cache.store(tileset, xtile, ytile, zoom, |mut f| {
            Tile::write_gz_to(&mut f, &tile.mvt_tile);
            Ok(())
        });
        tile.mvt_tile
    }
}


impl Config<MvtService> for MvtService {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        let pg = try!(PostgisInput::from_config(config));
        let grid = try!(Grid::from_config(config));
        let layers = try!(Layer::layers_from_config(config));
        let tilesets = config.lookup("tilesets")
                             .map_or_else(|| Vec::new(),
                                          |tilesets| {
            let mut sets = Vec::new();
            for (tileset, layerarray) in tilesets.as_table().unwrap() {
                let layers: Vec<String> = layerarray.as_slice().unwrap().iter().map(|l| l.as_str().unwrap().to_string() ).collect();
                debug!("Adding tileset {} {:?}", tileset, layers);
                sets.push(Tileset{name: tileset.clone(), layers: layers});
            }
            sets
        });
        let cache = try!(Tilecache::from_config(config));
        Ok(MvtService {input: pg, grid: grid,
                       layers: layers, tilesets: tilesets, cache: cache})
    }
    fn gen_config() -> String {
        let mut config = String::new();
        config.push_str(TOML_SERVICES);
        config.push_str(&Datasource::gen_config());
        config.push_str(&Grid::gen_config());
        config.push_str(&Layer::gen_config());
        config.push_str(TOML_TILESETS);
        config.push_str(&Tilecache::gen_config());
        config
    }
    fn gen_runtime_config(&self) -> String {
        let mut config = String::new();
        config.push_str(TOML_SERVICES);
        config.push_str(&self.input.gen_runtime_config());
        config.push_str(&self.grid.gen_runtime_config());
        for layer in &self.layers {
            config.push_str(&layer.gen_runtime_config());
        }
        config.push_str(TOML_TILESETS);
        config.push_str(&self.cache.gen_runtime_config());
        config
    }
}


const TOML_SERVICES: &'static str = r#"# t-rex configuration

[services]
mvt = true
"#;

const TOML_TILESETS: &'static str = r#"
[tilesets]
# Multiple layers in one vector tile
#tilesetname = ["layer1","layer2"]
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
    let mut layers = vec![Layer::new("points")];
    layers[0].table_name = Some(String::from("ne_10m_populated_places"));
    layers[0].geometry_field = Some(String::from("wkb_geometry"));
    layers[0].geometry_type = Some(String::from("POINT"));
    layers[0].query_limit = Some(1);
    let service = MvtService {input: pg, grid: grid, layers: layers,
                              tilesets: Vec::new(), cache: Tilecache::Nocache(Nocache)};

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

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let metadata = service.get_metadata("admin_0_countries");
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

[[layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#fid_field = "id"
#query = "SELECT name,wkb_geometry FROM mytable"

[tilesets]
# Multiple layers in one vector tile
#tilesetname = ["layer1","layer2"]

#[cache.file]
#base = "/tmp/mvtcache"
"#;
    println!("{}", &MvtService::gen_config());
    assert_eq!(expected, &MvtService::gen_config());
}
