//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
extern crate toml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate fallible_iterator;
extern crate postgis;
extern crate protobuf;
extern crate flate2;
extern crate pbr;

pub mod core;
pub mod datasource;
pub mod mvt;
pub mod service;
pub mod cache;

use std::env;
