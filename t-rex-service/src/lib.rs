//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

#[cfg(feature = "with-gdal")]
extern crate t_rex_gdal;

pub mod datasources;
pub mod metadata;
pub mod mvt_service;
#[cfg(test)]
mod mvt_service_test;
mod qgs_reader;
pub use qgs_reader::read_qgs;
