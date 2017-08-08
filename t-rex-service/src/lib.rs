//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate pbr;

extern crate t_rex_core;

use t_rex_core::core;
use t_rex_core::datasource;
use t_rex_core::cache;
use t_rex_core::mvt;
use t_rex_core::service;

pub mod mvt_service;
#[cfg(test)]
mod mvt_service_test;
