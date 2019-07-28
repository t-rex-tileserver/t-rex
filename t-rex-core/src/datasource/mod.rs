//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod datasource;
pub mod postgis_ds;
#[cfg(test)]
mod postgis_test;

pub use self::datasource::{DatasourceInput, DummyDatasource};
pub use self::postgis_ds::PostgisDatasource;
