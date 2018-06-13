//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

extern crate clap;
#[macro_use]
extern crate log;
extern crate actix;
extern crate actix_web;
extern crate futures;
extern crate open;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

extern crate t_rex_core;
extern crate t_rex_service;

use t_rex_core::{cache, core, datasource, service};
use t_rex_service::{datasource_type, mvt_service, read_qgs};

pub mod server;
