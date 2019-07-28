//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate tile_grid;

use t_rex_core::{cache, core, datasource, service};
use t_rex_service::{datasources, mvt_service, read_qgs};

mod runtime_config;
mod server;
mod static_files;

pub use crate::runtime_config::*;
pub use crate::server::webserver;
