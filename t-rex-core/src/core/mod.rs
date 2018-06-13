//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
mod enum_serializer;
pub mod config;
pub mod feature;
pub mod geom;
pub mod grid;
pub mod layer;
pub mod screen;

pub use self::config::{parse_config, read_config, ApplicationCfg, Config};

#[cfg(test)]
mod config_test;
#[cfg(test)]
mod geom_test;
#[cfg(test)]
mod grid_test;
#[cfg(test)]
mod layer_test;
