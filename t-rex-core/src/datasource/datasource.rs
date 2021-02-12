//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::DatasourceCfg;
use crate::core::feature::Feature;
use crate::core::layer::Layer;
use crate::core::Config;
use tile_grid::Extent;
use tile_grid::Grid;

pub trait DatasourceType {
    /// New instance with connected pool
    fn connected(&self) -> Self;
    fn detect_layers(&self, detect_geometry_types: bool) -> Vec<Layer>;
    /// Return column field names and Rust compatible type conversion - without geometry column
    fn detect_data_columns(&self, layer: &Layer, sql: Option<&String>) -> Vec<(String, String)>;
    fn layer_extent(&self, layer: &Layer, grid_srid: i32) -> Option<Extent>;
    fn prepare_queries(&mut self, tileset: &str, layer: &Layer, grid_srid: i32);
    /// Projected extent
    fn reproject_extent(
        &self,
        extent: &Extent,
        dest_srid: i32,
        src_srid: Option<i32>,
    ) -> Option<Extent>;
    /// Retrieve features of one layer. Return feature count.
    fn retrieve_features<F>(
        &self,
        tileset: &str,
        layer: &Layer,
        extent: &Extent,
        zoom: u8,
        grid: &Grid,
        read: F,
    ) -> u64
    where
        F: FnMut(&dyn Feature);
}

#[derive(Clone)]
pub struct DummyDatasource;

impl DatasourceType for DummyDatasource {
    fn connected(&self) -> DummyDatasource {
        unimplemented!();
    }
    fn detect_layers(&self, _detect_geometry_types: bool) -> Vec<Layer> {
        unimplemented!();
    }
    fn detect_data_columns(&self, _layer: &Layer, _sql: Option<&String>) -> Vec<(String, String)> {
        unimplemented!();
    }
    fn reproject_extent(
        &self,
        _extent: &Extent,
        _dest_srid: i32,
        _src_srid: Option<i32>,
    ) -> Option<Extent> {
        unimplemented!();
    }
    fn layer_extent(&self, _layer: &Layer, _grid_srid: i32) -> Option<Extent> {
        unimplemented!();
    }
    fn prepare_queries(&mut self, _tileset: &str, _layer: &Layer, _grid_srid: i32) {}
    fn retrieve_features<F>(
        &self,
        _tileset: &str,
        _layer: &Layer,
        _extent: &Extent,
        _zoom: u8,
        _grid: &Grid,
        _read: F,
    ) -> u64
    where
        F: FnMut(&dyn Feature),
    {
        0
    }
}

impl DummyDatasource {
    pub fn new(_: &str) -> DummyDatasource {
        DummyDatasource {}
    }
}

impl<'a> Config<'a, DatasourceCfg> for DummyDatasource {
    fn from_config(_ds_cfg: &DatasourceCfg) -> Result<Self, String> {
        Ok(DummyDatasource {})
    }
    fn gen_config() -> String {
        "".to_string()
    }
    fn gen_runtime_config(&self) -> String {
        "".to_string()
    }
}
