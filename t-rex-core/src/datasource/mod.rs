//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

mod datasource;
mod postgis_ds;
mod postgis_fields;
#[cfg(test)]
mod postgis_test;

pub use self::datasource::{DatasourceType, DummyDatasource};
pub use self::postgis_ds::PostgisDatasource;
