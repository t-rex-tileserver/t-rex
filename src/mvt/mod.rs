//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod tile;
#[cfg_attr(rustfmt, rustfmt_skip)]
pub mod vector_tile; // protoc --rust_out . vector_tile.proto
pub mod geom_encoder;
#[cfg(test)]
mod test_tile;
#[cfg(test)]
mod test_geom_encoder;
