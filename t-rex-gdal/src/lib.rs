#[macro_use]
extern crate log;

pub mod gdal_ds;
#[cfg(test)]
mod gdal_ds_test;

pub fn gdal_version() -> String {
    gdal::version::version_info("RELEASE_NAME")
}
