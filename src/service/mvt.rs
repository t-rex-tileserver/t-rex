//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{Datasource, DatasourceInput};
use datasource::PostgisInput;
use core::grid::{Grid, Extent, ExtentInt};
use core::layer::Layer;
use core::Config;
use mvt::tile::Tile;
use mvt::vector_tile;
use cache::{Cache, Tilecache};
use std::path::Path;
use std::fs::{self, File};
use toml;
use serde_json;
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

type JsonResult = Result<serde_json::Value, serde_json::error::Error>;

impl MvtService {
    pub fn connect(&mut self) {
        self.input = self.input.connected();
    }
    fn get_tileset(&self, name: &str) -> Vec<&Layer> {
        let tileset = self.tilesets.iter().find(|t| t.name == name);
        match tileset {
            Some(set) => set.layers.iter().map(|l| l).collect(),
            None => Vec::new(),
        }
    }
    /// Service metadata for backend web application
    pub fn get_mvt_metadata(&self) -> JsonResult {
        #[derive(Serialize)]
        struct MvtInfo {
            tilesets: Vec<TilesetInfo>,
        }
        #[derive(Serialize)]
        struct TilesetInfo {
            name: String,
            tilejson: String,
            tileurl: String,
            layers: Vec<LayerInfo>,
            supported: bool,
        }
        #[derive(Serialize)]
        struct LayerInfo {
            name: String,
            geometry_type: Option<String>,
        }

        let mut tileset_infos: Vec<TilesetInfo> = self.tilesets
            .iter()
            .map(|set| {
                let layerinfos = set.layers
                    .iter()
                    .map(|l| {
                             LayerInfo {
                                 name: l.name.clone(),
                                 geometry_type: l.geometry_type.clone(),
                             }
                         })
                    .collect();
                let supported = set.layers
                    .iter()
                    .any(|l| {
                             let geom_type =
                                 l.geometry_type.clone().unwrap_or("UNKNOWN".to_string());
                             ["POINT", "LINESTRING", "POLYGON"].contains(&(&geom_type as &str))
                         });
                TilesetInfo {
                    name: set.name.clone(),
                    tilejson: format!("{}.json", set.name),
                    tileurl: format!("/{}/{{z}}/{{x}}/{{y}}.pbf", set.name),
                    layers: layerinfos,
                    supported: supported,
                }
            })
            .collect();
        tileset_infos.sort_by_key(|ti| ti.name.clone());
        let mvt_info = MvtInfo { tilesets: tileset_infos };
        serde_json::to_value(mvt_info)
    }
    fn get_tilejson_metadata(&self, tileset: &str) -> JsonResult {
        Ok(json!({
            "id": tileset,
            "name": tileset,
            "description": tileset,
            "attribution": "",
            "format": "pbf",
            "version": "2.0.0",
            "scheme": "xyz",
            "bounds": [-180.0,-90.0,180.0,90.0], //TODO: bbox from data
            "minzoom": 0,  //TODO: make configurable
            "maxzoom": 14,  //TODO: make configurable
            "center": [0.0, 0.0, 2], //TODO: make configurable
            "basename": tileset
        }))
    }
    fn get_tilejson_layers(&self, tileset: &str) -> JsonResult {
        let layers = self.get_tileset(tileset);
        let layers_metadata: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let meta = layer.metadata();
                let query = layer.query(layer.maxzoom());
                let mut meta_json = json!({
                "id": meta.get("id").unwrap(),
                "name": meta.get("name").unwrap(),
                "description": meta.get("description").unwrap(),
                "srs": meta.get("srs").unwrap(),
                "properties": {
                    "minzoom": layer.minzoom(),
                    "maxzoom": layer.maxzoom(),
                    "buffer-size": layer.buffer_size.unwrap_or(0)
                },
                "fields": {}
            });
                //insert fields
                let fields = self.input.detect_data_columns(&layer, query);
                for (ref field, _) in fields {
                    meta_json["fields"]
                        .as_object_mut()
                        .unwrap()
                        .insert(field.clone(), json!(""));
                }
                meta_json
            })
            .collect();
        Ok(json!(layers_metadata))
    }
    fn get_tilejson_vector_layers(&self, tileset: &str) -> JsonResult {
        let layers = self.get_tileset(tileset);
        let vector_layers: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let meta = layer.metadata();
                let query = layer.query(layer.maxzoom());
                let mut layer_json = json!({
                "id": meta.get("id").unwrap(),
                "description": meta.get("description").unwrap(),
                "minzoom": layer.minzoom(),
                "maxzoom": layer.maxzoom(),
                "fields": {}
            });
                //insert fields
                let fields = self.input.detect_data_columns(&layer, query);
                for (ref field, _) in fields {
                    layer_json["fields"]
                        .as_object_mut()
                        .unwrap()
                        .insert(field.clone(), json!(""));
                }
                layer_json
            })
            .collect();
        Ok(json!(vector_layers))
    }
    /// TileJSON metadata (https://github.com/mapbox/tilejson-spec)
    pub fn get_tilejson(&self, baseurl: &str, tileset: &str) -> JsonResult {
        let mut metadata = self.get_tilejson_metadata(tileset)?;
        let vector_layers = self.get_tilejson_vector_layers(tileset)?;
        let url = json!([
            format!("{}/{}/{{z}}/{{x}}/{{y}}.pbf", baseurl, tileset)
        ]);
        let mut obj = metadata.as_object_mut().unwrap();
        obj.insert("tiles".to_string(), url);
        obj.insert("vector_layers".to_string(), vector_layers);
        Ok(json!(obj))
    }
    /// MapboxGL Style JSON (https://www.mapbox.com/mapbox-gl-style-spec/)
    pub fn get_stylejson(&self, baseurl: &str, tileset: &str) -> JsonResult {
        let mut stylejson = json!({
            "version": 8,
            "name": "t-rex",
            "metadata": {
                "mapbox:autocomposite": false,
                "mapbox:type": "template",
                "maputnik:renderer": "mbgljs",
                "openmaptiles:version": "3.x"
            },
            "glyphs": format!("{}/fonts/{{fontstack}}/{{range}}.pbf", baseurl),
            "sources": {
                tileset: {
                    "url": format!("{}/{}.json", baseurl, tileset),
                    "type": "vector"
                }
            }
        });
        let background_layer = json!({
          "id": "background_",
          "type": "background",
          "paint": {
            "background-color": "rgba(255, 255, 255, 1)"
          }
        }); // TODO: from global style
        let layers = self.get_tileset(tileset);
        let mut layer_styles: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let mut layerjson = if let Some(ref style) = layer.style {
                    serde_json::from_str(&style).unwrap()
                } else {
                    json!({})
                };
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("id".to_string(), json!(layer.name));
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("source".to_string(), json!(tileset));
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("source-layer".to_string(), json!(layer.name));
                // TODO: support source-layer referencing other layers
                // Default paint type
                let default_type = if let Some(ref geomtype) = layer.geometry_type {
                        match &geomtype as &str {
                            "POINT" => "circle",
                            _ => "line",
                        }
                    } else {
                        "line"
                    }
                    .to_string();
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .entry("type".to_string())
                    .or_insert(json!(default_type));

                layerjson
            })
            .collect();
        layer_styles.insert(0, background_layer);
        // Insert layers in stylejson
        let mut obj = stylejson.as_object_mut().unwrap();
        obj.insert("layers".to_string(), json!(layer_styles));
        Ok(json!(obj))
    }

    /// MBTiles metadata.json
    pub fn get_mbtiles_metadata(&self, tileset: &str) -> JsonResult {
        let mut metadata = self.get_tilejson_metadata(tileset)?;
        metadata["bounds"] = json!(metadata["bounds"].to_string());
        metadata["center"] = json!(metadata["center"].to_string());
        let layers = self.get_tilejson_layers(tileset)?;
        let vector_layers = self.get_tilejson_vector_layers(tileset)?;
        let metadata_vector_layers = json!({
            "Layer": layers,
            "vector_layers": vector_layers
        });
        let mut obj = metadata.as_object_mut().unwrap();
        obj.insert("json".to_string(),
                   json!(metadata_vector_layers.to_string()));
        Ok(json!(obj))
    }
    /// Prepare datasource queries. Must be called before requesting tiles.
    pub fn prepare_feature_queries(&mut self) {
        for tileset in &self.tilesets {
            for layer in &tileset.layers {
                self.input.prepare_queries(&layer, self.grid.srid);
            }
        }
    }
    /// Create vector tile from input at x, y, z in TMS adressing scheme
    pub fn tile(&self, tileset: &str, xtile: u32, ytile: u32, zoom: u8) -> vector_tile::Tile {
        let extent = self.grid.tile_extent(xtile, ytile, zoom);
        debug!("MVT tile request {:?}", extent);
        let mut tile = Tile::new(&extent, 4096, true);
        for layer in self.get_tileset(tileset) {
            let mut mvt_layer = tile.new_layer(layer);
            self.input
                .retrieve_features(&layer,
                                   &extent,
                                   zoom,
                                   &self.grid,
                                   |feat| { tile.add_feature(&mut mvt_layer, feat); });
            tile.add_layer(mvt_layer);
        }
        tile.mvt_tile
    }
    /// Fetch or create vector tile from input at x, y, z
    pub fn tile_cached(&self,
                       tileset: &str,
                       xtile: u32,
                       ytile: u32,
                       zoom: u8,
                       _gzip: bool)
                       -> Vec<u8> {
        // Reverse y for XYZ scheme (TODO: protocol instead of CRS dependent?)
        let y = if self.grid.srid == 3857 {
            self.grid.ytile_from_xyz(ytile, zoom)
        } else {
            ytile
        };
        let path = format!("{}/{}/{}/{}.pbf", tileset, zoom, xtile, ytile);

        let mut tile: Option<Vec<u8>> = None;
        self.cache
            .read(&path, |mut f| {
                let mut data = Vec::new();
                let _ = f.read_to_end(&mut data);
                tile = Some(data);
            });
        if tile.is_some() {
            //TODO: unzip if gzip == false
            return tile.unwrap();
        }

        let mvt_tile = self.tile(tileset, xtile, y, zoom);

        let mut tilegz = Vec::new();
        Tile::write_gz_to(&mut tilegz, &mvt_tile);
        let _ = self.cache.write(&path, &tilegz);

        //TODO: return unzipped if gzip == false
        tilegz
    }
    fn progress_bar(&self, msg: &str, limits: &ExtentInt) -> ProgressBar<Stdout> {
        let tiles = (limits.maxx as u64 - limits.minx as u64) *
                    (limits.maxy as u64 - limits.miny as u64);
        let mut pb = ProgressBar::new(tiles);
        pb.message(msg);
        //pb.set_max_refresh_rate(Some(Duration::from_millis(200)));
        pb.show_speed = false;
        pb.show_percent = false;
        pb.show_time_left = false;
        pb
    }
    /// Populate tile cache
    pub fn generate(&self,
                    tileset_name: Option<&str>,
                    minzoom: Option<u8>,
                    maxzoom: Option<u8>,
                    extent: Option<Extent>,
                    nodes: Option<u8>,
                    nodeno: Option<u8>,
                    progress: bool,
                    ignore_cache: bool) {
        self.init_cache();
        let minzoom = minzoom.unwrap_or(0);
        let maxzoom = maxzoom.unwrap_or(self.grid.maxzoom());
        let extent = extent.unwrap_or(self.grid.tile_extent(0, 0, 0));
        let nodes = nodes.unwrap_or(1) as u64;
        let nodeno = nodeno.unwrap_or(0) as u64;
        let mut tileno: u64 = 0;
        debug!("tile limits: {:?}", extent);
        let limits = self.grid.tile_limits(extent, 0);
        for tileset in &self.tilesets {
            if tileset_name.is_some() && tileset_name.unwrap() != &tileset.name {
                continue;
            }
            if progress {
                println!("Generating tileset '{}'...", tileset.name);
            }
            for zoom in minzoom..(maxzoom + 1) {
                if zoom > self.grid.maxzoom() {
                    warn!("Zoom level exceeds maximal zoom level of grid ({}) - skipping", self.grid.maxzoom());
                    continue;
                }
                let ref limit = limits[zoom as usize];
                debug!("level {}: {:?}", zoom, limit);
                let mut pb = self.progress_bar(&format!("Level {}: ", zoom), &limit);
                if progress {
                    pb.tick();
                }
                for xtile in limit.minx..limit.maxx {
                    for ytile in limit.miny..limit.maxy {
                        let skip = tileno % nodes != nodeno;
                        tileno += 1;
                        if skip {
                            continue;
                        }

                        // store in xyz schema. TODO: make configurable
                        let y = self.grid.ytile_from_xyz(ytile, zoom);
                        let path = format!("{}/{}/{}/{}.pbf", &tileset.name, zoom, xtile, y);

                        if !self.cache.exists(&path) || ignore_cache {
                            // Entry doesn't exist, or we're ignoring it, so generate it
                            let mvt_tile =
                                self.tile(&tileset.name, xtile as u32, ytile as u32, zoom);
                            let mut tilegz = Vec::new();
                            Tile::write_gz_to(&mut tilegz, &mvt_tile);
                            let _ = self.cache.write(&path, &tilegz);
                        }

                        if progress {
                            pb.inc();
                        }
                    }
                }
            }
        }
        if progress {
            println!("");
        }
    }
    pub fn init_cache(&self) {
        if let Tilecache::Filecache(ref fc) = self.cache {
            info!("Tile cache directory: {}", fc.basepath);
            // Write metadata.json for each tileset
            for tileset in &self.tilesets {
                let path = Path::new(&fc.basepath).join(&tileset.name);
                fs::create_dir_all(&path).unwrap();
                let f = File::create(&path.join("metadata.json")).unwrap();
                let _ = serde_json::to_writer(f,
                                              &self.get_mbtiles_metadata(&tileset.name).unwrap());
            }
        }
    }
}


impl Tileset {
    pub fn tilesets_from_config(config: &toml::Value) -> Result<Vec<Self>, String> {
        config
            .get("tileset")
            .ok_or("Missing configuration entry [[tileset]]".to_string())
            .and_then(|tarr| {
                          tarr.as_array()
                              .ok_or("Array type for [[tileset]] entry expected".to_string())
                      })
            .and_then(|tilesets| {
                          Ok(tilesets
                                 .iter()
                                 .map(|tileset| Tileset::from_config(tileset).unwrap())
                                 .collect())
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
        let name = config
            .get("name")
            .ok_or("Missing configuration entry name in [[tileset]]".to_string())
            .and_then(|val| {
                          val.as_str()
                              .ok_or("tileset.name entry is not a string".to_string())
                      })
            .map(|v| v.to_string());
        let layers = try!(Layer::layers_from_config(config));
        name.and_then(|n| {
                          Ok(Tileset {
                                 name: n,
                                 layers: layers,
                             })
                      })
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
        Ok(MvtService {
               input: pg,
               grid: grid,
               tilesets: tilesets,
               cache: cache,
           })
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
