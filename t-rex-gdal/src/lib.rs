#[macro_use]
extern crate log;

mod gdal_ds;
#[cfg(test)]
mod gdal_ds_test;
mod gdal_fields;

pub use self::gdal_ds::GdalDatasource;
pub use self::gdal_fields::ogr_layer_name;

pub fn gdal_version() -> String {
    gdal::version::version_info("RELEASE_NAME")
}
