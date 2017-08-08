//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;
use core::grid::Extent;
use core::grid::Grid;
use core::feature::Feature;


pub trait DatasourceInput {
    /// New instance with connected pool
    fn connected(&self) -> Self;
    /// Return column field names and Rust compatible type conversion - without geometry column
    fn detect_data_columns(&self, layer: &Layer, sql: Option<&String>) -> Vec<(String, String)>;
    fn prepare_queries(&mut self, layer: &Layer, grid_srid: i32);
    /// Projected extent
    fn extent_from_wgs84(&self, extent: &Extent, dest_srid: i32) -> Option<Extent>;
    fn retrieve_features<F>(&self,
                            layer: &Layer,
                            extent: &Extent,
                            zoom: u8,
                            grid: &Grid,
                            read: F)
        where F: FnMut(&Feature);
}

pub struct DummyDatasource;

impl DatasourceInput for DummyDatasource {
    fn connected(&self) -> DummyDatasource {
        unimplemented!();
    }
    fn detect_data_columns(&self, _layer: &Layer, _sql: Option<&String>) -> Vec<(String, String)> {
        unimplemented!();
    }
    fn extent_from_wgs84(&self, _extent: &Extent, _dest_srid: i32) -> Option<Extent> {
        unimplemented!();
    }
    fn prepare_queries(&mut self, _layer: &Layer, _grid_srid: i32) {}
    fn retrieve_features<F>(&self,
                            _layer: &Layer,
                            _extent: &Extent,
                            _zoom: u8,
                            _grid: &Grid,
                            _read: F)
        where F: FnMut(&Feature)
    {
    }
}
