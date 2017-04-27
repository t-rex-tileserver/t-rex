//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
mod enum_serializer;
pub mod geom;
pub mod screen;
pub mod grid;
pub mod layer;
pub mod feature;
pub mod config;

pub use self::config::{Config, read_config, parse_config};

#[cfg(test)]
mod test_geom;
#[cfg(test)]
mod test_grid;
#[cfg(test)]
mod test_layer;
#[cfg(test)]
mod test_config;
