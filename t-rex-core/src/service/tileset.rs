//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::grid::Extent;
use core::layer::Layer;
use core::config::Config;
use core::config::TilesetCfg;


/// Collection of layers in one MVT
pub struct Tileset {
    pub name: String,
    pub extent: Option<Extent>,
    pub layers: Vec<Layer>,
}

pub static WORLD_EXTENT: Extent = Extent {
    minx: -180.0,
    miny: -90.0,
    maxx: 180.0,
    maxy: 90.0,
};

impl Tileset {
    pub fn minzoom(&self) -> u8 {
        0 // TODO: from layers or config?
    }
    pub fn maxzoom(&self) -> u8 {
        22 // TODO: from layers or config?
    }
    pub fn get_extent(&self) -> &Extent {
        self.extent.as_ref().unwrap_or(&WORLD_EXTENT)
    }
    pub fn get_center(&self) -> (f64, f64) {
        let ext = self.get_extent();
        (ext.maxx - (ext.maxx - ext.minx) / 2.0, ext.maxy - (ext.maxy - ext.miny) / 2.0)
    }
    pub fn get_start_zoom(&self) -> u8 {
        2 // TODO: from config
    }
}

impl<'a> Config<'a, TilesetCfg> for Tileset {
    fn from_config(tileset_cfg: &TilesetCfg) -> Result<Self, String> {
        let layers = tileset_cfg
            .layers
            .iter()
            .map(|layer| Layer::from_config(layer).unwrap())
            .collect();
        Ok(Tileset {
               name: tileset_cfg.name.clone(),
               extent: tileset_cfg.extent.clone(),
               layers: layers,
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
