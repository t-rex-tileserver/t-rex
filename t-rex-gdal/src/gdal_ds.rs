//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::gdal_fields::*;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::vector::Geometry;
use gdal::Dataset;
use std::collections::BTreeMap;
use std::path::Path;
use t_rex_core::core::config::DatasourceCfg;
use t_rex_core::core::feature::Feature;
use t_rex_core::core::layer::Layer;
use t_rex_core::core::Config;
use t_rex_core::datasource::DatasourceType;
use tile_grid::Extent;
use tile_grid::Grid;

#[derive(Clone)]
pub struct GdalDatasource {
    pub path: String,
    // We don't store the Dataset, because we need mut access for getting layers
    /// SpatialRef WKT for layers which need CoordTransform
    geom_transform: BTreeMap<String, String>,
}

impl GdalDatasource {
    pub fn new(path: &str) -> GdalDatasource {
        GdalDatasource {
            path: path.to_string(),
            geom_transform: BTreeMap::new(),
        }
    }
}

impl DatasourceType for GdalDatasource {
    /// New instance with connected pool
    fn connected(&self) -> GdalDatasource {
        GdalDatasource {
            path: self.path.clone(),
            geom_transform: BTreeMap::new(),
        }
    }
    fn detect_layers(&self, _detect_geometry_types: bool) -> Vec<Layer> {
        let mut layers: Vec<Layer> = Vec::new();
        let dataset = Dataset::open(Path::new(&self.path)).unwrap();
        for gdal_layer in dataset.layers() {
            let name = gdal_layer.name();
            // Create a layer for each geometry field
            for (n, field) in gdal_layer.defn().geom_fields().enumerate() {
                let mut layer = Layer::new(&name);
                layer.table_name = if n == 0 {
                    Some(name.clone())
                } else {
                    Some(format!("{}_{}", &name, n))
                };
                layer.geometry_field = Some(field.name());
                layer.geometry_type = geom_type_name(field.field_type());
                let srs = field.spatial_ref().unwrap();
                if let Ok(epsg) = srs.auth_code() {
                    layer.srid = Some(epsg)
                }
                layers.push(layer)
            }
        }
        layers
    }
    /// Return column field names and Rust compatible type conversion - without geometry column
    fn detect_data_columns(&self, _layer: &Layer, _sql: Option<&String>) -> Vec<(String, String)> {
        Vec::new() //TODO
    }
    /// Projected extent
    fn reproject_extent(
        &self,
        extent: &Extent,
        dest_srid: i32,
        src_srid: Option<i32>,
    ) -> Option<Extent> {
        let ext_srid = src_srid.unwrap_or(4326);
        transform_extent(extent, ext_srid, dest_srid).ok()
    }
    fn layer_extent(&self, layer: &Layer, grid_srid: i32) -> Option<Extent> {
        let dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer_name = layer.table_name.as_ref().unwrap();
        let ogr_layer = dataset.layer_by_name(layer_name).unwrap();
        let extent = match ogr_layer.get_extent() {
            Err(e) => {
                warn!("Layer '{}': Unable to get extent: {:?}", layer.name, e);
                None
            }
            Ok(extent) => Some(Extent {
                minx: extent.MinX,
                miny: extent.MinY,
                maxx: extent.MaxX,
                maxy: extent.MaxY,
            }),
        };

        let grid_sref = match sref(grid_srid as u32) {
            Err(e) => {
                error!("Unable to get grid spatial reference: {:?}", e);
                return None;
            }
            Ok(sref) => sref,
        };

        let layer_sref = geom_spatialref(&ogr_layer, layer.geometry_field.as_ref());
        let src_sref = match layer_sref {
            Some(ref sref) if !layer.no_transform => sref,
            _ => &grid_sref,
        };

        let wgs84_sref = match sref(4326) {
            Err(e) => {
                warn!("Unable to get EPSG:4326 spatial reference: {:?}", e);
                return None;
            }
            Ok(sref) => sref,
        };

        match extent {
            Some(extent) => match transform_extent_sref(&extent, src_sref, &wgs84_sref) {
                Ok(extent) => Some(extent),
                Err(e) => {
                    error!("Unable to transform {:?}: {:?}", extent, e);
                    None
                }
            },
            None => None,
        }
    }
    fn prepare_queries(&mut self, _tileset: &str, layer: &Layer, grid_srid: i32) {
        if !Path::new(&self.path).exists() {
            warn!(
                "Layer '{}': Can't open dataset '{}'",
                layer.name, &self.path
            );
            // We continue, because GDAL also supports HTTP adresses
        }
        let dataset = Dataset::open(Path::new(&self.path));
        if let Err(ref err) = dataset {
            error!("Layer '{}': Error opening dataset: '{}'", layer.name, err);
            return;
        }
        let dataset = dataset.unwrap();
        if layer.table_name.is_none() {
            error!("Layer '{}': table_name missing", layer.name);
            return;
        }
        let layer_name = layer.table_name.as_ref().unwrap();
        let ogr_layer = dataset.layer_by_name(layer_name);
        if ogr_layer.is_err() {
            error!(
                "Layer '{}': Can't find dataset layer '{}'",
                layer.name, layer_name
            );
            return;
        }
        let ogr_layer = ogr_layer.unwrap();

        let grid_sref = match sref(grid_srid as u32) {
            Err(e) => {
                error!("Unable to get grid spatial reference: {:?}", e);
                return;
            }
            Ok(sref) => sref,
        };
        if !layer.no_transform {
            let layer_sref = geom_spatialref(&ogr_layer, layer.geometry_field.as_ref());
            if let Some(ref sref) = layer_sref {
                info!(
                    "Layer '{}': Reprojecting geometry to SRID {}",
                    layer.name, grid_srid
                );
                if CoordTransform::new(sref, &grid_sref).is_err() {
                    error!(
                        "Layer '{}': Couldn't setup CoordTransform for reprojecting geometry to SRID {}",
                        layer.name, grid_srid
                    );
                } else {
                    // We don't store prepared CoordTransform because CoordTransform is
                    // not Sync and cannot be shared between threads safely
                    self.geom_transform
                        .insert(layer.name.clone(), sref.to_wkt().unwrap());
                }
            } else {
                warn!("Layer '{}': Couldn't detect spatialref", layer.name);
            }
        }

        if layer.simplify {
            if layer.geometry_type != Some("POINT".to_string()) {
                warn!(
                    "Layer '{}': Simplification not supported for GDAL layers",
                    layer.name
                );
            }
        }
        if layer.buffer_size.is_some() {
            if layer.geometry_type != Some("POINT".to_string()) {
                warn!(
                    "Layer '{}': Clipping with buffer_size not supported for GDAL layers",
                    layer.name
                );
            }
        }
    }
    fn retrieve_features<F>(
        &self,
        _tileset: &str,
        layer: &Layer,
        extent: &Extent,
        zoom: u8,
        grid: &Grid,
        mut read: F,
    ) -> u64
    where
        F: FnMut(&dyn Feature),
    {
        let dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer_name = layer.table_name.as_ref().unwrap();
        debug!("retrieve_features layer: {}", layer_name);
        let mut ogr_layer = dataset.layer_by_name(layer_name).unwrap();

        let mut bbox_extent = if let Some(pixels) = layer.buffer_size {
            let pixel_width = grid.pixel_width(zoom);
            let buf = f64::from(pixels) * pixel_width;
            Extent {
                minx: extent.minx - buf,
                miny: extent.miny - buf,
                maxx: extent.maxx + buf,
                maxy: extent.maxy + buf,
            }
        } else {
            extent.clone()
        };

        // CoordTransform for features
        let mut transformation = None;
        if let Some(ref wkt) = self.geom_transform.get(&layer.name) {
            let grid_sref = sref(grid.srid as u32).unwrap();
            let layer_sref = SpatialRef::from_wkt(wkt).unwrap();
            // Spatial filter must be in layer SRS
            let bbox_tr = CoordTransform::new(&grid_sref, &layer_sref).unwrap();
            match transform_extent_tr(&bbox_extent, &bbox_tr) {
                Ok(extent) => bbox_extent = extent,
                Err(e) => {
                    error!("Unable to transform {:?}: {:?}", bbox_extent, e);
                    return 0;
                }
            }
            transformation = CoordTransform::new(&layer_sref, &grid_sref).ok();
        }
        let bbox = Geometry::bbox(
            bbox_extent.minx,
            bbox_extent.miny,
            bbox_extent.maxx,
            bbox_extent.maxy,
        )
        .unwrap();
        ogr_layer.set_spatial_filter(&bbox);

        let ogr_layer_for_defn = dataset.layer_by_name(layer_name).unwrap();
        let fields_defn = ogr_layer_for_defn.defn().fields().collect::<Vec<_>>();
        let mut cnt = 0;
        let query_limit = layer.query_limit.unwrap_or(0);
        for feature in ogr_layer.features() {
            let feat = VectorFeature {
                layer: layer,
                fields_defn: &fields_defn,
                grid_srid: grid.srid,
                transform: transformation.as_ref(),
                feature: &feature,
            };
            read(&feat);
            cnt += 1;
            if cnt == query_limit as u64 {
                info!(
                    "Features of layer {} limited to {} (tile query_limit reached, zoom level {})",
                    layer.name, cnt, zoom
                );
                break;
            }
        }
        cnt
    }
}

/// Projected extent
fn transform_extent(
    extent: &Extent,
    src_srid: i32,
    dest_srid: i32,
) -> Result<Extent, gdal::errors::GdalError> {
    let sref_in = sref(src_srid as u32)?;
    let sref_out = sref(dest_srid as u32)?;
    transform_extent_sref(extent, &sref_in, &sref_out)
}

const WKT_WSG84_LON_LAT: &str = r#"GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AXIS["Lon",EAST],AXIS["Lat",NORTH],AUTHORITY["EPSG","4326"]]"#;

fn sref(srid: u32) -> Result<SpatialRef, gdal::errors::GdalError> {
    if srid == 4326 {
        // Return WGS84 in traditional GIS axis order
        // See https://github.com/OSGeo/gdal/blob/master/gdal/doc/source/development/rfc/rfc73_proj6_wkt2_srsbarn.rst
        SpatialRef::from_wkt(WKT_WSG84_LON_LAT)
    } else {
        SpatialRef::from_epsg(srid)
    }
}

/// Projected extent
fn transform_extent_sref(
    extent: &Extent,
    src_sref: &SpatialRef,
    dest_sref: &SpatialRef,
) -> Result<Extent, gdal::errors::GdalError> {
    let transform = CoordTransform::new(src_sref, dest_sref)?;
    transform_extent_tr(extent, &transform)
}

/// Projected extent
fn transform_extent_tr(
    extent: &Extent,
    transformation: &CoordTransform,
) -> Result<Extent, gdal::errors::GdalError> {
    let xs = &mut [extent.minx, extent.maxx];
    let ys = &mut [extent.miny, extent.maxy];
    transformation.transform_coords(xs, ys, &mut [0.0, 0.0])?;
    Ok(Extent {
        minx: *xs.get(0).unwrap(),
        miny: *ys.get(0).unwrap(),
        maxx: *xs.get(1).unwrap(),
        maxy: *ys.get(1).unwrap(),
    })
}

impl<'a> Config<'a, DatasourceCfg> for GdalDatasource {
    fn from_config(ds_cfg: &DatasourceCfg) -> Result<Self, String> {
        Ok(GdalDatasource::new(ds_cfg.path.as_ref().unwrap()))
    }

    fn gen_config() -> String {
        let toml = r#"
[[datasource]]
name = "ds"
# Dataset specification (http://gdal.org/ogr_formats.html)
path = "<filename-or-connection-spec>"
"#;
        toml.to_string()
    }
    fn gen_runtime_config(&self) -> String {
        format!(
            r#"
[[datasource]]
path = "{}"
"#,
            self.path
        )
    }
}
