//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::datasources::{Datasource, Datasources};
use pbr::ProgressBar;
use percent_encoding::percent_decode;
use serde_json;
use std::cmp;
use std::io::{stderr, Stderr, Stdout};
use std::time::Instant;
use t_rex_core::cache::{Cache, Tilecache};
use t_rex_core::core::layer::Layer;
use t_rex_core::core::stats::Statistics;
use t_rex_core::core::{ApplicationCfg, Config};
use t_rex_core::datasource::DatasourceType;
use t_rex_core::mvt::tile::Tile;
use t_rex_core::mvt::vector_tile;
use t_rex_core::service::tileset::{Tileset, WORLD_EXTENT};
use tile_grid::{extent_wgs84_to_merc, Extent, ExtentInt, Grid, GridIterator};
use tokio::task;

/// Mapbox Vector Tile Service
#[derive(Clone)]
pub struct MvtService {
    pub datasources: Datasources,
    pub grid: Grid,
    pub tilesets: Vec<Tileset>,
    pub cache: Tilecache,
}

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
    pub(crate) fn ds(&self, layer: &Layer) -> Option<&Datasource> {
        self.datasources.datasource(&layer.datasource)
    }
    pub(crate) fn get_tileset(&self, name: &str) -> Option<&Tileset> {
        // URL decode tileset names from http requests
        let dec_name = percent_decode(name.as_bytes()).decode_utf8().unwrap();
        self.tilesets.iter().find(|t| t.name == dec_name)
    }
    /// Get layers (as reference) of given tileset
    pub(crate) fn get_tileset_layers(&self, name: &str) -> Vec<&Layer> {
        match self.get_tileset(name) {
            Some(set) => set.layers.iter().map(|l| l).collect(),
            None => Vec::new(),
        }
    }
    /// Prepare datasource queries. Must be called before requesting tiles.
    pub fn prepare_feature_queries(&mut self) {
        for tileset in &self.tilesets {
            for layer in &tileset.layers {
                let ds = self.datasources.datasource_mut(&layer.datasource);
                if ds.is_none() {
                    error!("Datasource of layer `{}` not found", layer.name);
                    continue;
                }
                ds.unwrap()
                    .prepare_queries(&tileset.name, &layer, self.grid.srid);
            }
        }
    }
    /// Create vector tile from input at x, y, z in TMS adressing scheme
    pub fn tile(
        &self,
        tileset: &str,
        xtile: u32,
        ytile: u32,
        zoom: u8,
        mut stats: Option<&mut Statistics>,
    ) -> vector_tile::Tile {
        let extent = self.grid.tile_extent(xtile, ytile, zoom);
        debug!(
            "{}/{}/{}/{} retrieving with {:?}",
            tileset, zoom, xtile, ytile, extent
        );
        let mut tile = Tile::new(&extent, true);
        for layer in self.get_tileset_layers(tileset) {
            if zoom >= layer.minzoom() && zoom <= layer.maxzoom(self.grid.maxzoom()) {
                let mut mvt_layer = tile.new_layer(layer);
                let now = Instant::now();
                let num_features = self.ds(&layer).unwrap().retrieve_features(
                    tileset,
                    &layer,
                    &extent,
                    zoom,
                    &self.grid,
                    |feat| {
                        tile.add_feature(&mut mvt_layer, feat);
                    },
                );
                let elapsed = now.elapsed();
                if let Some(ref mut stats) = stats {
                    stats.add(
                        format!("tile_ms.{}.{}.{}", tileset, layer.name, zoom),
                        elapsed.as_secs() * 1000 + elapsed.subsec_millis() as u64,
                    );
                    stats.add(
                        format!("feature_count.{}.{}.{}", tileset, layer.name, zoom),
                        num_features as u64,
                    );
                }
                debug!(
                    "{}/{}/{}/{} layer {}: {} features",
                    tileset, zoom, xtile, ytile, layer.name, num_features
                );
                if num_features > 0 {
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
        gzip: bool,
        stats: Option<&mut Statistics>,
    ) -> Option<Vec<u8>> {
        // Reverse y for XYZ scheme (TODO: protocol instead of CRS dependent?)
        let y = if self.grid.srid == 3857 {
            self.grid.ytile_from_xyz(ytile, zoom)
        } else {
            ytile
        };
        let path = format!("{}/{}/{}/{}.pbf", tileset, zoom, xtile, ytile);

        let ts = self
            .get_tileset(tileset)
            .expect(&format!("Tileset '{}' not found", tileset));

        if zoom < ts.minzoom() || zoom > ts.maxzoom() {
            return None;
        }

        let mut tile: Option<Vec<u8>> = None;
        if ts.is_cachable_at(zoom) {
            self.cache.read(&path, |f| {
                let mut data = Vec::new();
                let _ = f.read_to_end(&mut data);
                tile = Some(data);
            });
        } else {
            debug!(
                "Cache : read ignored for tileset {} at zoom {}",
                ts.name, zoom
            );
        }

        // Return tile from cache
        if let Some(tilegz) = tile {
            return Some(Tile::tile_content(tilegz, gzip));
        }

        // Request tile and write into cache
        let mvt_tile = self.tile(tileset, xtile, y, zoom, stats);
        // Spec: A Vector Tile SHOULD contain at least one layer.
        if mvt_tile.get_layers().len() > 0 {
            let tilegz = Tile::tile_bytevec_gz(&mvt_tile);
            if ts.is_cachable_at(zoom) {
                if let Err(ioerr) = self.cache.write(&path, &tilegz) {
                    error!("Error writing {}: {}", path, ioerr);
                }
            } else {
                debug!(
                    "Cache : write ignored for tileset {} at zoom {}",
                    ts.name, zoom
                );
            }
            Some(Tile::tile_content(tilegz, gzip))
        } else {
            // We don't save empty tiles
            // When serving from file cache return 204 No Content
            // Nginx: try_files $uri = 204;
            debug!("{} - Skipping empty tile", path);
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
    /// Projected extent in grid SRS
    pub fn extent_from_input_extent(&self, extent: &Extent, extent_srid: Option<i32>) -> Extent {
        // TODO: use proj4 (directly)
        let extent_srid_unwrapped = extent_srid.unwrap_or(4326);

        if self.grid.srid == 3857 && extent_srid_unwrapped == 4326 {
            // shortcut for Web Mercator
            extent_wgs84_to_merc(extent)
        } else {
            let ds = self.datasources.default().unwrap();
            if self.grid.srid == extent_srid_unwrapped {
                extent.clone()
            } else {
                ds.reproject_extent(extent, self.grid.srid, extent_srid)
                    .expect(&format!(
                        "Error transforming {:?} to SRID {}",
                        extent, self.grid.srid
                    ))
            }
        }
    }
    /// Seed tile cache
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
        extent_srid: Option<i32>,
    ) {
        let rt = tokio::runtime::Runtime::new().expect("Couldn't initialize tokio runtime");
        self.init_cache();
        let nodes = nodes.unwrap_or(1) as u64;
        let nodeno = nodeno.unwrap_or(0) as u64;

        for tileset in &self.tilesets {
            if tileset_name.is_some() && tileset_name.unwrap() != &tileset.name {
                continue;
            }
            if progress {
                println!("Generating tileset '{}'...", tileset.name);
            }

            // Convert extent to grid SRS
            let input_extent = extent.as_ref().or(tileset.extent.as_ref());
            debug!("input extent: {:?}", input_extent);
            let ext_proj = match input_extent {
                // (-180 -90) throws error when projecting
                Some(ext_wgs84) if *ext_wgs84 != WORLD_EXTENT => {
                    self.extent_from_input_extent(ext_wgs84, extent_srid)
                }
                _ => {
                    warn!("Building cache for the full globe, please fill in the tileset extent");
                    self.grid.tile_extent(0, 0, 0)
                }
            };
            debug!("tile limits: {:?}", ext_proj);

            let tolerance = 0;
            let limits = self.grid.tile_limits(ext_proj, tolerance);

            let ts_minzoom = cmp::max(tileset.minzoom(), minzoom.unwrap_or(0));
            let ts_maxzoom = *[
                tileset.maxzoom(),
                maxzoom.unwrap_or(99),
                self.grid.maxzoom(),
            ]
            .iter()
            .min()
            .unwrap_or(&22);
            if minzoom.is_some() && minzoom.unwrap() < ts_minzoom {
                warn!("Skipping zoom levels <{}", ts_minzoom);
            }
            if maxzoom.is_some() && maxzoom.unwrap() > ts_maxzoom {
                warn!("Skipping zoom levels >{}", ts_maxzoom);
            }
            rt.block_on(self.generate_tileset(
                limits,
                &tileset.name,
                ts_minzoom,
                ts_maxzoom,
                nodes,
                nodeno,
                progress,
                overwrite,
            ));
        }
        if progress {
            println!("");
        }
    }
    /// Seed tile cache for tileset
    async fn generate_tileset(
        &self,
        limits: Vec<ExtentInt>,
        tileset_name: &String,
        ts_minzoom: u8,
        ts_maxzoom: u8,
        nodes: u64,
        nodeno: u64,
        progress: bool,
        overwrite: bool,
    ) {
        // Keep a queue of tasks waiting for parallel async execution (size >= #cores).
        // libspatialite has a max connection limit of 64 for now. libspatialite (4.4.0) when
        // compiled on top of GEOS 3.5.0 is able to support an arbitrary number of threads
        let task_queue_size = cmp::min(num_cpus::get() * 2, 64);
        let mut tasks = Vec::with_capacity(task_queue_size);
        let griditer = GridIterator::new(ts_minzoom, ts_maxzoom, limits.clone());
        let mut tileno: u64 = 0;
        let mut pb = ProgressBar::new(0);
        let mut pb_z = !ts_minzoom;
        for (zoom, xtile, ytile) in griditer {
            if progress && zoom != pb_z {
                pb_z = zoom;
                let ref limit = limits[zoom as usize];
                debug!("level {}: {:?}", zoom, limit);
                pb = self.progress_bar(&format!("Level {}: ", zoom), &limit);
                pb.tick();
            }

            let skip = tileno % nodes != nodeno;
            tileno += 1;
            if skip {
                continue;
            }

            // Store Mercator tiles in xyz scheme, others in TMS scheme.
            let y = if self.grid.srid == 3857 {
                self.grid.ytile_from_xyz(ytile, zoom)
            } else {
                ytile
            };
            let path = format!("{}/{}/{}/{}.pbf", tileset_name, zoom, xtile, y);

            if overwrite || !self.cache.exists(&path) {
                // Entry doesn't exist, or overwrite is forced, so generate it
                let svc = self.clone();
                let cache = self.cache.clone();
                let tileset_name = tileset_name.clone();
                tasks.push(task::spawn(async move {
                    // rust-postgres starts its own Tokio runtime
                    // without spawn_blocking or block_in_place we get 'Cannot start a runtime from within a runtime'
                    let mvt_tile = task::spawn_blocking(move || {
                        svc.tile(&tileset_name, xtile as u32, ytile as u32, zoom, None)
                    })
                    .await
                    .unwrap();
                    if mvt_tile.get_layers().len() > 0 {
                        let tilegz = Tile::tile_bytevec_gz(&mvt_tile);
                        if let Err(ioerr) = cache.write(&path, &tilegz) {
                            error!("Error writing {}: {}", path, ioerr);
                        }
                    }
                }));
                if tasks.len() >= task_queue_size {
                    tasks = await_one_task(tasks).await;
                }
            }

            if progress {
                pb.inc();
            }
        }
        // Finish remaining tasks
        futures_util::future::join_all(tasks).await;
    }
    pub fn init_cache(&self) {
        info!("{}", &self.cache.info());
        for tileset in &self.tilesets {
            // :tileset.json
            let json = self
                .get_tilejson(&self.cache.baseurl(), &tileset.name)
                .unwrap();
            let _ = self.cache.write(
                &format!("{}.json", &tileset.name),
                &serde_json::to_vec(&json).unwrap(),
            );

            // :tileset.style.json
            let json = self
                .get_stylejson(&self.cache.baseurl(), &tileset.name)
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
    fn progress_bar_drilldown(&self, zoomlevels: u8, points: u64) -> ProgressBar<Stderr> {
        let numtiles = zoomlevels as u64 * points;
        let mut pb = ProgressBar::on(stderr(), numtiles);
        pb.message("Tile ");
        pb.show_speed = false;
        pb.show_percent = false;
        pb.show_time_left = false;
        pb
    }
    /// Get statistics from drilldown
    pub fn drilldown(
        &self,
        tileset_name: Option<&str>,
        minzoom: Option<u8>,
        maxzoom: Option<u8>,
        points: Vec<f64>,
        progress: bool,
    ) -> Statistics {
        let mut stats = Statistics::new();
        for tileset in &self.tilesets {
            if tileset_name.is_some() && tileset_name.unwrap() != &tileset.name {
                continue;
            }

            let ts_minzoom = cmp::max(tileset.minzoom(), minzoom.unwrap_or(0));
            let ts_maxzoom = *[
                tileset.maxzoom(),
                maxzoom.unwrap_or(99),
                self.grid.maxzoom(),
            ]
            .iter()
            .min()
            .unwrap_or(&22);

            let mut pb =
                self.progress_bar_drilldown(ts_maxzoom - ts_minzoom + 1, points.len() as u64 / 2);

            for point in points.chunks(2) {
                // Convert point to extent in grid SRS
                let ext_wgs84 = Extent {
                    minx: point[0],
                    miny: point[1],
                    maxx: point[0],
                    maxy: point[1],
                };
                let ext_proj = self.extent_from_input_extent(&ext_wgs84, None);
                debug!("point in grid SRS: {:?}", ext_proj);

                let tolerance = 0;
                let limits = self.grid.tile_limits(ext_proj, tolerance);
                for zoom in ts_minzoom..=ts_maxzoom {
                    let ref limit = limits[zoom as usize];
                    debug!("level {}: {:?}", zoom, limit);
                    let xtile = limit.minx;
                    let ytile = limit.miny;
                    let mvt_tile = self.tile(
                        &tileset.name,
                        xtile as u32,
                        ytile as u32,
                        zoom,
                        Some(&mut stats),
                    );
                    stats.add(
                        format!("tile_bytes.{}.total.{}", &tileset.name, zoom),
                        Tile::size(&mvt_tile) as u64,
                    );
                    if progress {
                        pb.inc();
                    }
                }
            }
        }
        if progress {
            eprintln!("");
        }
        stats
    }
    fn gen_layer_runtime_config(&self, layer: &Layer, grid_srid: i32) -> String {
        let ds = self.ds(layer).unwrap();
        let mut lines = vec!["\n[[tileset]]".to_string()];
        lines.push(format!(r#"name = "{}""#, layer.name));
        if layer.no_transform {
            if let Some(layer_srid) = layer.srid {
                if let Some(ext) = ds.layer_extent(layer, layer_srid) {
                    lines.push(format!(
                        r#"# Real extent: [{:.5}, {:.5}, {:.5}, {:.5}]"#,
                        ext.minx, ext.miny, ext.maxx, ext.maxy
                    ));
                }
            }
        }
        if let Some(ext) = ds.layer_extent(layer, grid_srid) {
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
                let query = pg.build_query_sql_template(layer);
                cfg.push_str(&format!("#sql = \"\"\"{}\"\"\"\n", query))
            }
        }
        cfg
    }
}

async fn await_one_task<T>(tasks: Vec<task::JoinHandle<T>>) -> Vec<task::JoinHandle<T>> {
    match futures_util::future::select_all(tasks).await {
        // Ignoring all errors
        (_result, _index, remaining) => remaining,
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
            datasources,
            grid,
            tilesets,
            cache,
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
                config.push_str(&self.gen_layer_runtime_config(layer, self.grid.srid));
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
