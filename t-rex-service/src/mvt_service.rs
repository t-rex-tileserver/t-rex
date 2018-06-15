//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use cache::{Cache, Tilecache};
use core::grid::{Extent, ExtentInt, Grid};
use core::layer::Layer;
use core::ApplicationCfg;
use core::Config;
use datasource::DatasourceInput;
use datasource_type::Datasource;
use datasource_type::Datasources;
use mvt::tile::Tile;
use mvt::vector_tile;
use pbr::ProgressBar;
use percent_encoding::percent_decode;
use serde_json;
use service::tileset::{Tileset, WORLD_EXTENT};
use std::io::Stdout;

/// Mapbox Vector Tile Service
pub struct MvtService {
    pub datasources: Datasources,
    pub grid: Grid,
    pub tilesets: Vec<Tileset>,
    pub cache: Tilecache,
}

type JsonResult = Result<serde_json::Value, serde_json::error::Error>;

impl MvtService {
    /// Connect all datasources
    // Needed before calling methods on PostGIS datasources like prepare_feature_queries or get_mbtiles_metadata
    // TODO: connect automatically when needed
    pub fn connect(&mut self) {
        let mut datasources = Datasources::new();
        datasources.default = self.datasources.default.clone();
        for (name, ds) in &self.datasources.datasources {
            datasources.add(&name, ds.connected());
        }
        datasources.setup();
        self.datasources = datasources;
    }
    fn ds(&self, layer: &Layer) -> Option<&Datasource> {
        self.datasources.datasource(&layer.datasource)
    }
    fn get_tileset(&self, name: &str) -> Option<&Tileset> {
        // URL decode tileset names from http requests
        let dec_name = percent_decode(name.as_bytes()).decode_utf8().unwrap();
        self.tilesets.iter().find(|t| t.name == dec_name)
    }
    /// Get layers (as reference) of given tileset
    fn get_tileset_layers(&self, name: &str) -> Vec<&Layer> {
        match self.get_tileset(name) {
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
            bounds: [f64; 4],
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
                    .map(|l| LayerInfo {
                        name: l.name.clone(),
                        geometry_type: l.geometry_type.clone(),
                    })
                    .collect();
                let supported = set.layers.iter().any(|l| {
                    let geom_type = l.geometry_type.clone().unwrap_or("UNKNOWN".to_string());
                    ["POINT", "LINESTRING", "POLYGON"].contains(&(&geom_type as &str))
                });
                let ext = set.get_extent();
                TilesetInfo {
                    name: set.name.clone(),
                    tilejson: format!("{}.json", set.name),
                    tileurl: format!("/{}/{{z}}/{{x}}/{{y}}.pbf", set.name),
                    bounds: [ext.minx, ext.miny, ext.maxx, ext.maxy],
                    layers: layerinfos,
                    supported: supported,
                }
            })
            .collect();
        tileset_infos.sort_by_key(|ti| ti.name.clone());
        let mvt_info = MvtInfo {
            tilesets: tileset_infos,
        };
        serde_json::to_value(mvt_info)
    }
    fn get_tilejson_metadata(&self, tileset: &str) -> JsonResult {
        let ts = self.get_tileset(tileset).unwrap();
        let ext = ts.get_extent();
        let center = ts.get_center();
        let zoom = ts.get_start_zoom();
        Ok(json!({
            "id": tileset,
            "name": tileset,
            "description": tileset,
            "attribution": ts.attribution(),
            "format": "pbf",
            "version": "2.0.0",
            "scheme": "xyz",
            "bounds": [ext.minx,
                       ext.miny,
                       ext.maxx,
                       ext.maxy],
            "minzoom": ts.minzoom(),
            "maxzoom": ts.maxzoom(),
            "center": [center.0, center.1, zoom],
            "basename": tileset
        }))
    }
    fn get_tilejson_layers(&self, tileset: &str) -> JsonResult {
        let layers = self.get_tileset_layers(tileset);
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
                let fields = self.ds(&layer).unwrap().detect_data_columns(&layer, query);
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
    /// TileJSON MVT vector layer extension (https://github.com/mapbox/tilejson-spec/issues/14)
    fn get_tilejson_vector_layers(&self, tileset: &str) -> JsonResult {
        let layers = self.get_tileset_layers(tileset);
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
                let fields = self.ds(&layer).unwrap().detect_data_columns(&layer, query);
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
        let url = json!([format!("{}/{}/{{z}}/{{x}}/{{y}}.pbf", baseurl, tileset)]);
        let obj = metadata.as_object_mut().unwrap();
        obj.insert("tiles".to_string(), url);
        obj.insert("vector_layers".to_string(), vector_layers);
        Ok(json!(obj))
    }
    /// MapboxGL Style JSON (https://www.mapbox.com/mapbox-gl-style-spec/)
    pub fn get_stylejson(&self, baseurl: &str, tileset: &str) -> JsonResult {
        // TODO: add minZoom/maxZoom for vector source.
        // Difference between setting the maxZoom for a source, and setting
        // the maxZoom for a layer:
        // (https://github.com/mapbox/mapbox-gl-native/issues/9863#issuecomment-325615680)
        //
        // Source maxZoom controls from which zoom levels tiles are loaded. If
        // your custom tile source only has tiles up to z14, please set
        // maxZoom: 14 so that Mapbox GL doesn't attempt to load z15/z16/...
        // tiles. As per the style specification, the default value of maxZoom
        // is 22.
        // https://www.mapbox.com/mapbox-gl-js/style-spec/#sources-vector-maxzoom
        //
        // Layer maxZoom controls when the layer is displayed depending on the
        // zoom. This is independent of the source zoom level, e.g. we can
        // show z14 tiles when zoomed to 16. If you specify a maxZoom of 14,
        // the layer won't be shown at all when the zoom level is >= 14, even
        // if there are still tiles available.
        // https://www.mapbox.com/mapbox-gl-js/style-spec/#layer-maxzoom
        let mut stylejson = json!({
            "version": 8,
            "name": "t-rex",
            "metadata": {
                "mapbox:autocomposite": false,
                "mapbox:type": "template",
                "maputnik:renderer": "mbgljs"
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
        let layers = self.get_tileset_layers(tileset);
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
                }.to_string();
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
        let obj = stylejson.as_object_mut().unwrap();
        obj.insert("layers".to_string(), json!(layer_styles));
        Ok(json!(obj))
    }

    /// MBTiles metadata.json (https://github.com/mapbox/mbtiles-spec/blob/master/1.3/spec.md)
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
        let obj = metadata.as_object_mut().unwrap();
        obj.insert(
            "json".to_string(),
            json!(metadata_vector_layers.to_string()),
        );
        Ok(json!(obj))
    }
    /// Prepare datasource queries. Must be called before requesting tiles.
    pub fn prepare_feature_queries(&mut self) {
        for tileset in &self.tilesets {
            for layer in &tileset.layers {
                let ds = self.datasources
                    .datasource_mut(&layer.datasource)
                    .expect(&format!("Datasource of layer `{}` not found", layer.name));
                ds.prepare_queries(&layer, self.grid.srid);
            }
        }
    }
    /// Create vector tile from input at x, y, z in TMS adressing scheme
    pub fn tile(&self, tileset: &str, xtile: u32, ytile: u32, zoom: u8) -> vector_tile::Tile {
        let extent = self.grid.tile_extent(xtile, ytile, zoom);
        debug!("MVT tile request {:?}", extent);
        let mut tile = Tile::new(&extent, true);
        for layer in self.get_tileset_layers(tileset) {
            if zoom >= layer.minzoom() && zoom <= layer.maxzoom() {
                let mut mvt_layer = tile.new_layer(layer);
                self.ds(&layer).unwrap().retrieve_features(
                    &layer,
                    &extent,
                    zoom,
                    &self.grid,
                    |feat| {
                        tile.add_feature(&mut mvt_layer, feat);
                    },
                );
                if mvt_layer.get_features().len() > 0 {
                    tile.add_layer(mvt_layer);
                }
            }
        }
        tile.mvt_tile
    }
    /// Fetch or create vector tile from input at x, y, z
    pub fn tile_cached(
        &self,
        tileset: &str,
        xtile: u32,
        ytile: u32,
        zoom: u8,
        _gzip: bool,
    ) -> Option<Vec<u8>> {
        // Reverse y for XYZ scheme (TODO: protocol instead of CRS dependent?)
        let y = if self.grid.srid == 3857 {
            self.grid.ytile_from_xyz(ytile, zoom)
        } else {
            ytile
        };
        let path = format!("{}/{}/{}/{}.pbf", tileset, zoom, xtile, ytile);

        let mut tile: Option<Vec<u8>> = None;
        self.cache.read(&path, |f| {
            let mut data = Vec::new();
            let _ = f.read_to_end(&mut data);
            tile = Some(data);
        });
        if tile.is_some() {
            //TODO: unzip if gzip == false
            return tile;
        }

        let mvt_tile = self.tile(tileset, xtile, y, zoom);
        let mut tilegz = Vec::new();
        Tile::write_gz_to(&mut tilegz, &mvt_tile);
        let _ = self.cache.write(&path, &tilegz);

        if mvt_tile.get_layers().len() > 0 {
            //TODO: return unzipped if gzip == false
            Some(tilegz)
        } else {
            None
        }
    }
    fn progress_bar(&self, msg: &str, limits: &ExtentInt) -> ProgressBar<Stdout> {
        let tiles =
            (limits.maxx as u64 - limits.minx as u64) * (limits.maxy as u64 - limits.miny as u64);
        let mut pb = ProgressBar::new(tiles);
        pb.message(msg);
        //pb.set_max_refresh_rate(Some(Duration::from_millis(200)));
        pb.show_speed = false;
        pb.show_percent = false;
        pb.show_time_left = false;
        pb
    }
    /// Projected extent in grid SRS from WGS84
    pub fn extent_from_wgs84(&self, extent: &Extent) -> Extent {
        // TODO: use proj4 (directly)
        // and maybe fast track for Web Mercator (see fn xy in grid_test)
        let ds = self.datasources.default().unwrap();
        ds.extent_from_wgs84(extent, self.grid.srid)
            .expect(&format!(
                "Error transforming {:?} to SRID {}",
                extent, self.grid.srid
            ))
    }
    /// Populate tile cache
    pub fn generate(
        &self,
        tileset_name: Option<&str>,
        minzoom: Option<u8>,
        maxzoom: Option<u8>,
        extent: Option<Extent>,
        nodes: Option<u8>,
        nodeno: Option<u8>,
        progress: bool,
        overwrite: bool,
    ) {
        self.init_cache();
        let minzoom = minzoom.unwrap_or(0);
        let maxzoom = maxzoom.unwrap_or(self.grid.maxzoom());
        let nodes = nodes.unwrap_or(1) as u64;
        let nodeno = nodeno.unwrap_or(0) as u64;
        let mut tileno: u64 = 0;
        for tileset in &self.tilesets {
            if tileset_name.is_some() && tileset_name.unwrap() != &tileset.name {
                continue;
            }
            if progress {
                println!("Generating tileset '{}'...", tileset.name);
            }

            // Convert extent to grid SRS
            let extent = extent.as_ref().or(tileset.extent.as_ref());
            debug!("wgs84 extent: {:?}", extent);
            let ext_proj = match extent {
                // (-180 -90) throws error when projecting
                Some(ext_wgs84) if *ext_wgs84 != WORLD_EXTENT => self.extent_from_wgs84(ext_wgs84),
                _ => {
                    warn!("Building cache for the full globe, please fill in the tileset extent");
                    self.grid.tile_extent(0, 0, 0)
                }
            };
            debug!("tile limits: {:?}", ext_proj);

            let tolerance = 0;
            let limits = self.grid.tile_limits(ext_proj, tolerance);
            for zoom in minzoom..maxzoom + 1 {
                if zoom > self.grid.maxzoom() {
                    warn!(
                        "Zoom level exceeds maximal zoom level of grid ({}) - skipping",
                        self.grid.maxzoom()
                    );
                    continue;
                }
                let ref limit = limits[zoom as usize];
                debug!("level {}: {:?}", zoom, limit);
                let mut pb = self.progress_bar(&format!("Level {}: ", zoom), &limit);
                if progress {
                    pb.tick();
                }
                for xtile in limit.minx..limit.maxx + 1 {
                    for ytile in limit.miny..limit.maxy + 1 {
                        let skip = tileno % nodes != nodeno;
                        tileno += 1;
                        if skip {
                            continue;
                        }

                        // store in xyz schema. TODO: make configurable
                        let y = self.grid.ytile_from_xyz(ytile, zoom);
                        let path = format!("{}/{}/{}/{}.pbf", &tileset.name, zoom, xtile, y);

                        if overwrite || !self.cache.exists(&path) {
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
        info!("{}", &self.cache.info());
        for tileset in &self.tilesets {
            // :tileset.json
            let json = self.get_tilejson(&self.cache.baseurl(), &tileset.name)
                .unwrap();
            let _ = self.cache.write(
                &format!("{}.json", &tileset.name),
                &serde_json::to_vec(&json).unwrap(),
            );

            // :tileset.style.json
            let json = self.get_stylejson(&self.cache.baseurl(), &tileset.name)
                .unwrap();
            let _ = self.cache.write(
                &format!("{}.style.json", &tileset.name),
                &serde_json::to_vec(&json).unwrap(),
            );

            // :tileset/metadata.json
            let json = self.get_mbtiles_metadata(&tileset.name).unwrap();
            let _ = self.cache.write(
                &format!("{}/metadata.json", &tileset.name),
                &serde_json::to_vec(&json).unwrap(),
            );
        }
    }
    fn gen_layer_runtime_config(&self, layer: &Layer) -> String {
        let ds = self.ds(layer).unwrap();
        let extent = ds.layer_extent(layer);
        let mut lines = vec!["\n[[tileset]]".to_string()];
        lines.push(format!(r#"name = "{}""#, layer.name));
        if let Some(ext) = extent {
            lines.push(format!(
                r#"extent = [{:.5}, {:.5}, {:.5}, {:.5}]"#,
                ext.minx, ext.miny, ext.maxx, ext.maxy
            ));
        } else {
            lines.push("#extent = [-180.0,-90.0,180.0,90.0]".to_string());
        }

        let mut cfg = lines.join("\n") + "\n";
        cfg.push_str(&layer.gen_runtime_config());
        if let &Datasource::Postgis(ref pg) = ds {
            if layer.query(0).is_none() {
                let query = pg.build_query_sql(layer, 3857, None, true).unwrap();
                // Remove quotes from column names for better readability
                cfg.push_str(&format!("#sql = \"\"\"{}\"\"\"\n", query.replace('"', "")))
            }
        }
        cfg
    }
}

impl<'a> Config<'a, ApplicationCfg> for MvtService {
    fn from_config(config: &ApplicationCfg) -> Result<Self, String> {
        let datasources = Datasources::from_config(config)?;
        let grid = Grid::from_config(&config.grid)?;
        let tilesets = config
            .tilesets
            .iter()
            .map(|ts_cfg| Tileset::from_config(ts_cfg).unwrap())
            .collect();
        let cache = Tilecache::from_config(&config)?;
        Ok(MvtService {
            datasources: datasources,
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
        config.push_str(&self.datasources.gen_runtime_config());
        config.push_str(&self.grid.gen_runtime_config());
        for tileset in &self.tilesets {
            for layer in &tileset.layers {
                config.push_str(&self.gen_layer_runtime_config(layer));
            }
        }
        config.push_str(&self.cache.gen_runtime_config());
        config
    }
}

const TOML_SERVICES: &'static str = r#"# t-rex configuration

[service.mvt]
viewer = true
"#;
