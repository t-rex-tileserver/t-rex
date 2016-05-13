//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::layer::Layer;
use core::grid::Extent;
use core::feature::Feature;


pub trait Datasource {
    fn retrieve_features<F>(&self, layer: &Layer, extent: &Extent, zoom: u16, mut read: F)
        where F : FnMut(&Feature);
}
