//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{Datasource,DatasourceInput};
use datasource::PostgisInput;
use core::grid::{Grid, Extent, ExtentInt};
use core::layer::Layer;
use core::Config;
use mvt::tile::Tile;
use mvt::vector_tile;
use cache::{Cache,Tilecache};
use std::path::Path;
use std::fs::{self,File};
use std::io::Write;
use toml;
use rustc_serialize::json::{self, Json, ToJson};
use pbr::ProgressBar;
use std::io::Stdout;


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
    pub fn connect(&mut self) {
        self.input = self.input.connected();
    }
    fn get_tileset(&self, name: &str) -> Vec<&Layer> {
        let tileset = self.tilesets.iter().find(|t| t.name == name);
        match tileset {
            Some(set) => set.layers.iter().map(|l| l).collect(),
            None => Vec::new()
        }
    }
    /// Service metadata for backend web application
    pub fn get_mvt_metadata(&self) -> Json {
        #[derive(RustcEncodable)]
        struct MvtInfo {
            tilesets: Vec<TilesetInfo>,
        }
        #[derive(RustcEncodable)]
        struct TilesetInfo {
            name: String,
            tilejson: String,
            tileurl: String,
            layers: Vec<LayerInfo>,
            supported: bool,
        }
        #[derive(RustcEncodable)]
        struct LayerInfo {
            name: String,
            geometry_type: Option<String>,
        }

        let mut tileset_infos: Vec<TilesetInfo> = self.tilesets.iter().map(|set| {
            let layerinfos = set.layers.iter().map(|l| {
                LayerInfo { name: l.name.clone(), geometry_type: l.geometry_type.clone() }
                }).collect();
            let supported = set.layers.iter().any(|l| {
                let geom_type = l.geometry_type.clone().unwrap_or("UNKNOWN".to_string());
                ["POINT","LINESTRING","POLYGON"].contains(&(&geom_type as &str))
            });
            TilesetInfo {
                name: set.name.clone(),
                tilejson: format!("{}.json", set.name),
                tileurl: format!("/{}/{{z}}/{{x}}/{{y}}.pbf", set.name),
                layers: layerinfos,
                supported: supported,
            }
        }).collect();
        tileset_infos.sort_by_key(|ti| ti.name.clone());
        let mvt_info = MvtInfo { tilesets: tileset_infos };
        let encoded = json::encode(&mvt_info).unwrap();
        Json::from_str(&encoded).unwrap()
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
            "center": [0.0, 0.0, 2],
            "basename": "{}"
        }}"#, tileset, tileset, tileset, tileset)).unwrap();
        let layers_metadata: Vec<(String,String)> = layers.iter().map(|layer| {
            let meta = layer.metadata();
            let query = layer.query(layer.maxzoom());
            let fields = self.input.detect_data_columns(&layer, query);
            let fields_json: Vec<String> = fields.iter().map(|&(ref f, _)| format!("\"{}\": \"\"", f)).collect();
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
    /// TileJSON metadata (https://github.com/mapbox/tilejson-spec)
    pub fn get_tilejson(&self, baseurl: &str, tileset: &str) -> String {
        let (mut metadata, _layers, vector_layers) = self.get_tilejson_infos(tileset);
        let mut obj = metadata.as_object_mut().unwrap();
        let url = Json::from_str(&format!("[\"{}/{}/{{z}}/{{x}}/{{y}}.pbf\"]", baseurl, tileset)).unwrap();
        obj.insert("tiles".to_string(), url);
        obj.insert("vector_layers".to_string(), vector_layers);
        obj.to_json().to_string()
    }
    /// MapboxGL Style JSON (https://www.mapbox.com/mapbox-gl-style-spec/)
    pub fn get_stylejson(&self, baseurl: &str, tileset: &str) -> String {
        let json = Json::from_str(&format!(r#"
        {{
            "version": 8,
            "name": "t-rex",
            "sources": {{
                "{}": {{
                    "url": "{}/{}.json",
                    "type": "vector"
                }}
            }},
            "layers": [
                {{
                    "id": "{}",
                    "type": "line",
                    "source": "{}",
                    "source-layer": "{}"
                }}
            ]
        }}"#, tileset, baseurl, tileset, tileset, tileset, tileset)).unwrap();
        json.to_string()
    }
    /// MBTiles metadata.json
    pub fn get_mbtiles_metadata(&self, tileset: &str) -> String {
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
    /// Prepare datasource queries. Must be called before requesting tiles.
    pub fn prepare_feature_queries(&mut self) {
        for tileset in &self.tilesets {
            for layer in &tileset.layers {
                self.input.prepare_queries(&layer, self.grid.srid);
            }
        }
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u8) -> vector_tile::Tile {
        let extent = if self.grid.srid == 3857 {
            self.grid.tile_extent_reverse_y(xtile, ytile, zoom)
        } else {
            self.grid.tile_extent(xtile, ytile, zoom)
        };
        debug!("MVT tile request {:?}", extent);
        let mut tile = Tile::new(&extent, 4096, true);
        for layer in self.get_tileset(tileset) {
            let mut mvt_layer = tile.new_layer(layer);
            self.input.retrieve_features(&layer, &extent, zoom, &self.grid, |feat| {
                tile.add_feature(&mut mvt_layer, feat);
            });
            tile.add_layer(mvt_layer);
        }
        tile.mvt_tile
    }
    /// Fetch or create vector tile from input at x, y, z
    pub fn tile_cached(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u8, _gzip: bool) -> Vec<u8> {
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
    fn progress_bar(&self, msg: &str, limits: &ExtentInt) -> ProgressBar<Stdout> {
        let tiles = (limits.maxx as u64-limits.minx as u64)*(limits.maxy as u64-limits.miny as u64);
        let mut pb = ProgressBar::new(tiles);
        pb.message(msg);
        //pb.set_max_refresh_rate(Some(Duration::from_millis(200)));
        pb.show_speed = false;
        pb.show_percent = false;
        pb.show_time_left = false;
        pb
    }
    /// Populate tile cache
    pub fn generate(&self, tileset_name: Option<&str>, minzoom: Option<u8>, maxzoom: Option<u8>,
                    extent: Option<Extent>, nodes: Option<u8>, nodeno: Option<u8>, progress: bool) {
        self.init_cache();
        let minzoom = minzoom.unwrap_or(0);
        let maxzoom = maxzoom.unwrap_or(self.grid.nlevels());
        let extent = extent.unwrap_or(self.grid.tile_extent(0, 0, 0));
        let nodes = nodes.unwrap_or(1) as u64;
        let nodeno = nodeno.unwrap_or(0) as u64;
        let mut tileno: u64 = 0;
        let limits = self.grid.tile_limits(extent, 0);
        for tileset in &self.tilesets {
            if tileset_name.is_some() &&
               tileset_name.unwrap() != &tileset.name {
                continue;
            }
            if progress { println!("Generating tileset '{}'...", tileset.name); }
            for zoom in minzoom..maxzoom {
                let ref limit = limits[zoom as usize];
                let mut pb = self.progress_bar(&format!("Level {}: ", zoom), &limit);
                if progress { pb.tick(); }
                for xtile in limit.minx..limit.maxx {
                    for ytile in limit.miny..limit.maxy {
                        let skip = tileno % nodes != nodeno;
                        tileno += 1;
                        if skip { continue; }

                        let mvt_tile = self.tile(&tileset.name, xtile, ytile, zoom);
                        let mut tilegz = Vec::new();
                        Tile::write_gz_to(&mut tilegz, &mvt_tile);
                        let path = format!("{}/{}/{}/{}.pbf", &tileset.name, zoom, xtile, ytile);
                        let _ = self.cache.write(&path, &tilegz);
                        if progress { pb.inc(); }
                    }
                }
            }
        }
        if progress { println!(""); }
    }
    pub fn init_cache(&self) {
        if let Tilecache::Filecache(ref fc) = self.cache {
            info!("Tile cache directory: {}", fc.basepath);
            // Write metadata.json for each tileset
            for tileset in &self.tilesets {
                let path = Path::new(&fc.basepath).join(&tileset.name);
                fs::create_dir_all(&path).unwrap();
                let mut f = File::create(&path.join("metadata.json")).unwrap();
                let _ = f.write_all(self.get_mbtiles_metadata(&tileset.name).as_bytes());
            }
        }
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
    pub fn gen_runtime_config_from_input(&self, input: &PostgisInput) -> String {
        let mut config = String::new();
        for layer in &self.layers {
            config.push_str(&layer.gen_runtime_config_from_input(input));
        }
        config
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
            config.push_str(&tileset.gen_runtime_config_from_input(&self.input));
        }
        config.push_str(&self.cache.gen_runtime_config());
        config
    }
}


const TOML_SERVICES: &'static str = r#"# t-rex configuration

[service.mvt]
viewer = true
"#;


#[cfg(test)] use cache::Nocache;

#[test]
#[ignore]
pub fn test_tile_query() {
    use std::env;

    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput::new(&val).connected()),
        Result::Err(_) => { panic!("DBCONN undefined") }
    }.unwrap();
    let grid = Grid::web_mercator();
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    layer.query_limit = Some(1);
    let tileset = Tileset{name: "points".to_string(), layers: vec![layer]};
    let mut service = MvtService {input: pg, grid: grid,
                              tilesets: vec![tileset], cache: Tilecache::Nocache(Nocache)};
    service.prepare_feature_queries();

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
                    tags: [
                        0,
                        0,
                        1,
                        1,
                        2,
                        2,
                        3,
                        3
                    ],
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
            keys: [
                "fid",
                "scalerank",
                "name",
                "pop_max"
            ],
            values: [
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        106
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        10
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: Some("Delemont"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        11315
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
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
pub fn test_mvt_metadata() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();

    let metadata = format!("{}", service.get_mvt_metadata().pretty());
    let expected = r#"{
  "tilesets": [
    {
      "layers": [
        {
          "geometry_type": "POINT",
          "name": "points"
        },
        {
          "geometry_type": "POLYGON",
          "name": "buildings"
        }
      ],
      "name": "osm",
      "supported": true,
      "tilejson": "osm.json",
      "tileurl": "/osm/{z}/{x}/{y}.pbf"
    }
  ]
}"#;
    println!("{}", metadata);
    assert_eq!(metadata, expected);
}

#[test]
#[ignore]
pub fn test_tilejson() {
    use core::read_config;
    use std::env;

    if env::var("DBCONN").is_err() {
        panic!("DBCONN undefined");
    }

    let config = read_config("src/test/example.cfg").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    service.prepare_feature_queries();

    let metadata = service.get_tilejson("http://127.0.0.1", "osm");
    let metadata = Json::from_str(&metadata).unwrap().pretty().to_string();
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": [
    -180.0,
    -90.0,
    180.0,
    90.0
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "maxzoom": 14,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "tiles": [
    "http://127.0.0.1/osm/{z}/{x}/{y}.pbf"
  ],
  "vector_layers": [
    {
      "description": "",
      "fields": {
        "fid": "",
        "name": "",
        "pop_max": "",
        "scalerank": ""
      },
      "id": "points",
      "maxzoom": 99,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "buildings",
      "maxzoom": 99,
      "minzoom": 0
    }
  ],
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
pub fn test_stylejson() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let json = service.get_stylejson("http://127.0.0.1", "osm");
    let json = Json::from_str(&json).unwrap().pretty().to_string();
    println!("{}", json);
    let expected= r#"{
  "layers": [
    {
      "id": "osm",
      "source": "osm",
      "source-layer": "osm",
      "type": "line"
    }
  ],
  "name": "t-rex",
  "sources": {
    "osm": {
      "type": "vector",
      "url": "http://127.0.0.1/osm.json"
    }
  },
  "version": 8
}"#;
    assert_eq!(json, expected);

    // Mapbox GL style experiments
    let configjson = json::encode(&config.lookup("tileset.0.layer.1.style").unwrap()).unwrap().replace("}{", "},{").replace("][", "],[");
    let configjson = Json::from_str(&configjson).unwrap().pretty().to_string();
    println!("{}", configjson);
    let expected= r##"[
  {
    "fill-color": {
      "stops": [
        {
          "in": 15.5,
          "out": "#f2eae2"
        },
        {
          "in": 16,
          "out": "#dfdbd7"
        }
      ]
    },
    "interactive": true,
    "type": "fill"
  },
  {
    "circle-color": [
      {
        "property": "temperature",
        "stops": [
          {
            "in": 0,
            "out": "blue"
          },
          {
            "in": 100,
            "out": "red"
          }
        ]
      }
    ],
    "fill-color": "#f2eae2",
    "fill-outline-color": "#dfdbd7",
    "fill-translate": {
      "stops": [
        {
          "in": 15,
          "out": [
            11
          ]
        },
        {
          "in": 16,
          "out": [
            -20
          ]
        }
      ]
    },
    "fillopacity": {
      "base": 1,
      "stops": [
        [
          150
        ],
        [
          161
        ]
      ]
    },
    "interactive": true,
    "type": "fill"
  }
]"##;
    assert_eq!(configjson, expected);
}

#[test]
#[ignore]
pub fn test_mbtiles_metadata() {
    use core::read_config;
    use std::env;

    if env::var("DBCONN").is_err() {
        panic!("DBCONN undefined");
    }

    let config = read_config("src/test/example.cfg").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    let metadata = service.get_mbtiles_metadata("osm");
    let metadata = Json::from_str(&metadata).unwrap().pretty().to_string();
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": [
    -180.0,
    -90.0,
    180.0,
    90.0
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "json": "{\"Layer\":[{\"description\":\"\",\"fields\":{\"fid\":\"\",\"name\":\"\",\"pop_max\":\"\",\"scalerank\":\"\"},\"id\":\"points\",\"name\":\"points\",\"properties\":{\"buffer-size\":0,\"maxzoom\":99,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"name\":\"buildings\",\"properties\":{\"buffer-size\":0,\"maxzoom\":99,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"}],\"vector_layers\":[{\"description\":\"\",\"fields\":{\"fid\":\"\",\"name\":\"\",\"pop_max\":\"\",\"scalerank\":\"\"},\"id\":\"points\",\"maxzoom\":99,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"maxzoom\":99,\"minzoom\":0}]}",
  "maxzoom": 14,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
pub fn test_gen_config() {
    let expected = r#"# t-rex configuration

[service.mvt]
viewer = true

[datasource]
type = "postgis"
# Connection specification (https://github.com/sfackler/rust-postgres#connecting)
url = "postgresql://user:pass@host/database"

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
#simplify = true
#buffer-size = 10
#[[tileset.layer.query]]
#minzoom = 0
#maxzoom = 22
#sql = "SELECT name,wkb_geometry FROM mytable"

#[cache.file]
#base = "/tmp/mvtcache"
"#;
    println!("{}", &MvtService::gen_config());
    assert_eq!(expected, &MvtService::gen_config());
}
