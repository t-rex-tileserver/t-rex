extern crate gdal;
extern crate gdal_sys;
#[macro_use]
extern crate log;
extern crate t_rex_core;

use t_rex_core::core;
use t_rex_core::datasource;

pub mod gdal_ds;
#[cfg(test)]
mod gdal_ds_test;
