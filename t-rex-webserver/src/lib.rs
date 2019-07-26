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

use t_rex_core::{cache, core, datasource, service};
use t_rex_service::{datasource_type, mvt_service, read_qgs};

pub mod server;
