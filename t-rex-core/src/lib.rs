//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

extern crate fallible_iterator;
extern crate flate2;
#[macro_use]
extern crate log;
extern crate postgis;
extern crate postgres;
extern crate postgres_native_tls;
extern crate protobuf;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate stats;
extern crate tile_grid;
extern crate toml;

pub mod cache;
pub mod core;
pub mod datasource;
pub mod mvt;
pub mod service;

use std::env;
