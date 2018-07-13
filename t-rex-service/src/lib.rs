//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

extern crate clap;
extern crate elementtree;
#[macro_use]
extern crate log;
extern crate pbr;
extern crate percent_encoding;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

extern crate t_rex_core;
#[cfg(feature = "with-gdal")]
extern crate t_rex_gdal;

use t_rex_core::cache;
use t_rex_core::core;
use t_rex_core::datasource;
use t_rex_core::mvt;
use t_rex_core::service;
#[cfg(feature = "with-gdal")]
use t_rex_gdal::gdal_ds;

pub mod datasource_type;
pub mod metadata;
pub mod mvt_service;
#[cfg(test)]
mod mvt_service_test;
mod qgs_reader;
pub use qgs_reader::read_qgs;
