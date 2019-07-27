//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
pub mod config;
pub mod feature;
pub mod geom;
mod gridcfg;
pub mod layer;
pub mod screen;
pub mod stats;

pub use self::config::{parse_config, read_config, ApplicationCfg, Config};

#[cfg(test)]
mod config_test;
#[cfg(test)]
mod geom_test;
#[cfg(test)]
mod gridcfg_test;
#[cfg(test)]
mod layer_test;
