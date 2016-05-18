//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::datasource::Datasource;
use datasource::postgis::PostgisInput;
use core::grid::{Extent,Grid};
use core::layer::Layer;
use mvt::tile::Tile;
use mvt::vector_tile;
use mvt::geom_encoder::EncodableGeom;


/// Collection of layers in one MVT
pub struct Topic {
    pub name: String,
    pub layers: Vec<String>,
}

/// Mapbox Vector Tile Service
pub struct MvtService {
    pub input: PostgisInput,
    pub grid: Grid,
    pub layers: Vec<Layer>,
    pub topics: Vec<Topic>,
}

impl MvtService {
    fn get_layers(&self, name: &str) -> Vec<&Layer> {
        let topic = self.topics.iter().find(|t| t.name == name);
        match topic {
            Some(_) => Vec::new(), //TODO: return corresponding layers
            None => {
                self.layers.iter().filter(|t| t.name == name).collect()
            }
        }
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let extent = self.grid.tile_extent_reverse_y(xtile, ytile, zoom);
        debug!("MVT tile request {:?}", extent);
        let mut tile = Tile::new(&extent, 4096, true);
        for layer in self.get_layers(topic).iter() {
            let mut mvt_layer = tile.new_layer(layer);
            self.input.retrieve_features(&layer, &extent, zoom, |feat| {
                tile.add_feature(&mut mvt_layer, feat);
            });
            tile.add_layer(mvt_layer);
        }
        tile.mvt_tile
    }
}

#[cfg(test)] use std::io::{self,Write};
#[cfg(test)] use std::env;

#[test]
pub fn test_tile_query() {
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
    let service = MvtService {input: pg, grid: grid, layers: layers, topics: Vec::new()};

    let mvt_tile = service.tile("points", 33, 22, 6);
    println!("{:#?}", mvt_tile);
    let expected = "Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some(\"points\"),
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
                    cached_size: Cell { value: 0 }
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
            cached_size: Cell { value: 0 }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell { value: 0 }
}";
    assert_eq!(expected, &*format!("{:#?}", mvt_tile));
}
