//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::Config;
use crate::core::config::{TilesetCacheCfg, TilesetCfg};
use crate::core::layer::Layer;
use tile_grid::Extent;

#[derive(Clone, Debug)]
pub struct CacheLimits {
    pub minzoom: u8,
    pub maxzoom: Option<u8>,
    pub no_cache: bool,
}

impl<'a> Config<'a, TilesetCacheCfg> for CacheLimits {
    fn from_config(cfg: &TilesetCacheCfg) -> Result<Self, String> {
        Ok(CacheLimits {
            minzoom: cfg.minzoom,
            maxzoom: cfg.maxzoom.clone(),
            no_cache: cfg.no_cache,
        })
    }
    fn gen_config() -> String {
        "".to_string()
    }
}

/// Collection of layers in one MVT
#[derive(Clone)]
pub struct Tileset {
    pub name: String,
    pub minzoom: Option<u8>,
    pub maxzoom: Option<u8>,
    pub attribution: Option<String>,
    pub extent: Option<Extent>,
    pub center: Option<(f64, f64)>,
    pub start_zoom: Option<u8>,
    pub layers: Vec<Layer>,
    pub cache_limits: Option<CacheLimits>,
}

pub static WORLD_EXTENT: Extent = Extent {
    minx: -180.0,
    miny: -90.0,
    maxx: 180.0,
    maxy: 90.0,
};

impl Tileset {
    pub fn minzoom(&self) -> u8 {
        self.minzoom
            .unwrap_or(self.layers.iter().map(|l| l.minzoom()).min().unwrap_or(0))
    }
    pub fn maxzoom(&self) -> u8 {
        self.maxzoom.unwrap_or(
            self.layers
                .iter()
                .map(|l| l.maxzoom(22))
                .max()
                .unwrap_or(22),
        )
    }
    pub fn attribution(&self) -> String {
        self.attribution.clone().unwrap_or("".to_string())
    }
    pub fn get_extent(&self) -> &Extent {
        self.extent.as_ref().unwrap_or(&WORLD_EXTENT)
    }
    pub fn get_center(&self) -> (f64, f64) {
        if self.center.is_none() {
            let ext = self.get_extent();
            (
                ext.maxx - (ext.maxx - ext.minx) / 2.0,
                ext.maxy - (ext.maxy - ext.miny) / 2.0,
            )
        } else {
            self.center.unwrap()
        }
    }
    pub fn get_start_zoom(&self) -> u8 {
        self.start_zoom.unwrap_or(2)
    }
    pub fn is_cachable_at(&self, zoom: u8) -> bool {
        match self.cache_limits {
            Some(ref cl) => !cl.no_cache && cl.minzoom <= zoom && cl.maxzoom.unwrap_or(99) >= zoom,
            None => true,
        }
    }
}

impl<'a> Config<'a, TilesetCfg> for Tileset {
    fn from_config(tileset_cfg: &TilesetCfg) -> Result<Self, String> {
        let layers = tileset_cfg
            .layers
            .iter()
            .map(|layer| Layer::from_config(layer).unwrap())
            .collect();
        let cache_limits: Option<CacheLimits> = match tileset_cfg.cache_limits {
            Some(ref cfg) => match CacheLimits::from_config(&cfg) {
                Ok(cl) => Some(cl),
                _ => None,
            },
            None => None,
        };
        let extent = match &tileset_cfg.extent {
            Some(cfg) => Some(Extent::from(cfg)),
            None => None,
        };
        Ok(Tileset {
            name: tileset_cfg.name.clone(),
            minzoom: tileset_cfg.minzoom.clone(),
            maxzoom: tileset_cfg.maxzoom.clone(),
            attribution: tileset_cfg.attribution.clone(),
            extent,
            center: tileset_cfg.center.clone(),
            start_zoom: tileset_cfg.start_zoom.clone(),
            layers: layers,
            cache_limits: cache_limits,
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

#[test]
fn test_zoom() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    let mut tileset = Tileset {
        name: "points".to_string(),
        minzoom: None,
        maxzoom: None,
        center: None,
        start_zoom: Some(3),
        attribution: None,
        extent: Some(Extent {
            minx: -179.58998,
            miny: -90.00000,
            maxx: 179.38330,
            maxy: 82.48332,
        }),
        layers: vec![layer],
        cache_limits: None,
    };

    assert_eq!(tileset.minzoom(), 0);
    assert_eq!(tileset.maxzoom(), 22);

    tileset.layers[0].maxzoom = Some(8);
    assert_eq!(tileset.maxzoom(), 8);

    tileset.layers[0].minzoom = Some(3);
    assert_eq!(tileset.minzoom(), 3);

    tileset.minzoom = Some(2);
    assert_eq!(tileset.minzoom(), 2);
}
