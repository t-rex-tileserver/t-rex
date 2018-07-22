//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::config::DatasourceCfg;
use core::feature::{Feature, FeatureAttr, FeatureAttrValType};
use core::geom::{self, GeometryType};
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use core::Config;
use datasource::DatasourceInput;
use gdal;
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::vector::{Dataset, FieldValue, Geometry, OGRwkbGeometryType};
use gdal_sys;
use std::collections::BTreeMap;
use std::path::Path;

pub struct GdalDatasource {
    pub path: String,
    // We don't store the Dataset, because we need mut access for getting layers
    // CoordTransform for all layers
    geom_transform: BTreeMap<String, Option<CoordTransform>>,
    // CoordTransform for all layers
    bbox_transform: BTreeMap<String, Option<CoordTransform>>,
}

impl GdalDatasource {
    pub fn new(path: &str) -> GdalDatasource {
        GdalDatasource {
            path: path.to_string(),
            geom_transform: BTreeMap::new(),
            bbox_transform: BTreeMap::new(),
        }
    }
}

trait ToGeo {
    fn to_geo(&self, srid: Option<i32>) -> GeometryType;
}

fn ogr_type_name(ogr_type: OGRwkbGeometryType::Type) -> String {
    use std::ffi::CStr;
    let rv = unsafe { gdal_sys::OGRGeometryTypeToName(ogr_type) };
    //_string(rv)
    let c_str = unsafe { CStr::from_ptr(rv) };
    c_str.to_string_lossy().into_owned()
}

impl ToGeo for Geometry {
    /// Convert OGR geomtry to t-rex EWKB geometry type (XY only)
    fn to_geo(&self, srid: Option<i32>) -> GeometryType {
        let geometry_type = self.geometry_type();

        let ring = |n: usize| {
            let ring = unsafe { self._get_geometry(n) };
            return match ring.to_geo(srid) {
                GeometryType::LineString(r) => r,
                _ => panic!("Expected to get a LineString"),
            };
        };

        match geometry_type {
            OGRwkbGeometryType::wkbPoint => {
                let (x, y, _) = self.get_point(0);
                GeometryType::Point(geom::Point {
                    x: x,
                    y: y,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbMultiPoint => {
                let point_count = self.geometry_count();
                let coords = (0..point_count)
                    .map(|n| match unsafe { self._get_geometry(n) }.to_geo(srid) {
                        GeometryType::Point(p) => p,
                        _ => panic!("Expected to get a Point"),
                    })
                    .collect();
                GeometryType::MultiPoint(geom::MultiPoint {
                    points: coords,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbLineString => {
                let coords = self.get_point_vec()
                    .iter()
                    .map(|&(x, y, _)| geom::Point {
                        x: x,
                        y: y,
                        srid: srid,
                    })
                    .collect();
                GeometryType::LineString(geom::LineString {
                    points: coords,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbMultiLineString => {
                let string_count = self.geometry_count();
                let strings = (0..string_count)
                    .map(|n| match unsafe { self._get_geometry(n) }.to_geo(srid) {
                        GeometryType::LineString(s) => s,
                        _ => panic!("Expected to get a LineString"),
                    })
                    .collect();
                GeometryType::MultiLineString(geom::MultiLineString {
                    lines: strings,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbPolygon => {
                let ring_count = self.geometry_count();
                let rings = (0..ring_count).map(|n| ring(n)).collect();
                GeometryType::Polygon(geom::Polygon {
                    rings: rings,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbMultiPolygon => {
                let string_count = self.geometry_count();
                let strings = (0..string_count)
                    .map(|n| match unsafe { self._get_geometry(n) }.to_geo(srid) {
                        GeometryType::Polygon(s) => s,
                        _ => panic!("Expected to get a Polygon"),
                    })
                    .collect();
                GeometryType::MultiPolygon(geom::MultiPolygon {
                    polygons: strings,
                    srid: srid,
                })
            }
            /* TODO:
            OGRwkbGeometryType::wkbGeometryCollection => {
                let item_count = self.geometry_count();
                let geometry_list = (0..item_count)
                    .map(|n| unsafe { self._get_geometry(n) }.to_geo(srid))
                    .collect();
                GeometryType::GeometryCollection(geom::GeometryCollection {
                                                     geometries: geometry_list,
                                                 })
            }
            */
            geom_type => panic!("Unsupported geometry type {}", &ogr_type_name(geom_type)),
        }
    }
}

/// Projected extent
fn transform_extent(
    extent: &Extent,
    src_srid: i32,
    dest_srid: i32,
) -> Result<Extent, gdal::errors::Error> {
    let sref_in = SpatialRef::from_epsg(src_srid as u32)?;
    let sref_out = SpatialRef::from_epsg(dest_srid as u32)?;
    transform_extent_sref(extent, &sref_in, &sref_out)
}

/// Projected extent
fn transform_extent_sref(
    extent: &Extent,
    src_sref: &SpatialRef,
    dest_sref: &SpatialRef,
) -> Result<Extent, gdal::errors::Error> {
    let transform = CoordTransform::new(src_sref, dest_sref)?;
    transform_extent_tr(extent, &transform)
}

/// Projected extent
fn transform_extent_tr(
    extent: &Extent,
    transformation: &CoordTransform,
) -> Result<Extent, gdal::errors::Error> {
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

pub fn ogr_layer_name(path: &str, id: isize) -> Result<String, gdal::errors::Error> {
    let mut dataset = Dataset::open(Path::new(path))?;
    let layer = dataset.layer(id)?;
    Ok(layer.name())
}

struct VectorFeature<'a> {
    layer: &'a Layer,
    fields_defn: &'a Vec<gdal::vector::Field<'a>>,
    grid_srid: i32,
    transform: Option<&'a CoordTransform>,
    feature: &'a gdal::vector::Feature<'a>,
}

impl<'a> Feature for VectorFeature<'a> {
    fn fid(&self) -> Option<u64> {
        self.layer.fid_field.as_ref().and_then(|fid| {
            let field_value = self.feature.field(&fid);
            match field_value {
                Ok(FieldValue::IntegerValue(v)) => Some(v as u64),
                _ => None,
            }
        })
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        let mut attrs = Vec::new();
        for (_i, field) in self.fields_defn.into_iter().enumerate() {
            let field_value = self.feature.field(&field.name()); //TODO: get by index
            let val = match field_value {
                Ok(FieldValue::StringValue(v)) => Some(FeatureAttrValType::String(v)),
                Ok(FieldValue::IntegerValue(v)) => Some(FeatureAttrValType::Int(v as i64)),
                Ok(FieldValue::RealValue(v)) => Some(FeatureAttrValType::Double(v)),
                Err(err) => {
                    warn!(
                        "Layer '{}' - skipping field '{}': {}",
                        self.layer.name,
                        field.name(),
                        err
                    );
                    None
                }
            };
            // match field.field_type {
            //    OGRFieldType::OFTString => {
            if let Some(val) = val {
                let fattr = FeatureAttr {
                    key: field.name(),
                    value: val,
                };
                attrs.push(fattr);
            };
        }
        attrs
    }
    fn geometry(&self) -> Result<GeometryType, String> {
        let ogrgeom = if let Some(ref field) = self.layer.geometry_field {
            self.feature.geometry_by_name(field).unwrap()
        } else {
            self.feature.geometry()
        };
        if let Some(ref transform) = self.transform {
            ogrgeom.transform_inplace(transform).unwrap();
        };
        Ok(ogrgeom.to_geo(Some(self.grid_srid)))
    }
}

impl DatasourceInput for GdalDatasource {
    /// New instance with connected pool
    fn connected(&self) -> GdalDatasource {
        GdalDatasource {
            path: self.path.clone(),
            geom_transform: BTreeMap::new(),
            bbox_transform: BTreeMap::new(),
        }
    }
    fn detect_layers(&self, _detect_geometry_types: bool) -> Vec<Layer> {
        let mut layers: Vec<Layer> = Vec::new();
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        for idx in 0..dataset.count() {
            let gdal_layer = dataset.layer(idx).unwrap();
            let name = gdal_layer.name();
            for (n, field) in gdal_layer.defn().geom_fields().enumerate() {
                let mut layer = Layer::new(&name);
                layer.table_name = if n == 0 {
                    Some(name.clone())
                } else {
                    Some(format!("{}_{}", &name, n))
                };
                layer.geometry_field = Some(field.name());
                //layer.geometry_type = Some("POINT".to_string()); //TODO
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
    fn extent_from_wgs84(&self, extent: &Extent, dest_srid: i32) -> Option<Extent> {
        transform_extent(extent, 4326, dest_srid).ok()
    }
    fn layer_extent(&self, layer: &Layer, grid_srid: i32) -> Option<Extent> {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer_name = layer.table_name.as_ref().unwrap();
        let ogr_layer = dataset.layer_by_name(layer_name).unwrap();
        let extent = match ogr_layer.get_extent(true) {
            Err(e) => {
                warn!("Layer '{}': Unable to get extent: {}", layer.name, e);
                None
            }
            Ok(extent) => Some(Extent {
                minx: extent.MinX,
                miny: extent.MinY,
                maxx: extent.MaxX,
                maxy: extent.MaxY,
            }),
        };

        let layer_sref = match ogr_layer.spatial_reference() {
            //FIXME: use layer.geom_field
            Err(e) => {
                warn!(
                    "Layer '{}': Unable to get spatial reference: {}",
                    layer.name, e
                );
                return None;
            }
            Ok(sref) => sref,
        };

        let grid_sref = match SpatialRef::from_epsg(grid_srid as u32) {
            Err(e) => {
                error!("Unable to get grid spatial reference: {}", e);
                return None;
            }
            Ok(sref) => sref,
        };

        let src_sref = if layer.no_transform {
            grid_sref
        } else {
            layer_sref
        };

        let wgs84_sref = match SpatialRef::from_epsg(4326) {
            Err(e) => {
                warn!("Unable to get EPSG:4326 spatial reference: {}", e);
                return None;
            }
            Ok(sref) => sref,
        };

        match extent {
            Some(extent) => match transform_extent_sref(&extent, &src_sref, &wgs84_sref) {
                Ok(extent) => Some(extent),
                Err(e) => {
                    error!("Unable to transform {:?}: {}", extent, e);
                    None
                }
            },
            None => None,
        }
    }
    fn prepare_queries(&mut self, layer: &Layer, grid_srid: i32) {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer_name = layer.table_name.as_ref().unwrap();
        let ogr_layer = dataset.layer_by_name(layer_name).unwrap();

        let layer_sref = layer
            .srid
            .map(|srid| SpatialRef::from_epsg(srid as u32))
            .unwrap_or_else(|| ogr_layer.spatial_reference()) //FIXME: use layer.geom_field
            .or_else(|_| SpatialRef::from_epsg(grid_srid as u32));
        let layer_sref = match layer_sref {
            Err(e) => {
                error!(
                    "Layer '{}': Unable to get spatial reference: {}",
                    layer.name, e
                );
                return;
            }
            Ok(sref) => sref,
        };
        let grid_sref = match SpatialRef::from_epsg(grid_srid as u32) {
            Err(e) => {
                error!("Unable to get grid spatial reference: {}", e);
                return;
            }
            Ok(sref) => sref,
        };
        let transform = if layer_sref != grid_sref && !layer.no_transform {
            info!(
                "Layer '{}': Reprojecting geometry to SRID {:?}",
                layer.name, grid_srid
            );
            CoordTransform::new(&layer_sref, &grid_sref).ok()
        } else {
            None
        };
        self.geom_transform.insert(layer.name.clone(), transform);
        let transform = if layer_sref != grid_sref && !layer.no_transform {
            CoordTransform::new(&grid_sref, &layer_sref).ok()
        } else {
            None
        };
        self.bbox_transform.insert(layer.name.clone(), transform);

        if layer.simplify {
            warn!(
                "Layer '{}': Simplification not supported for GDAL layers",
                layer.name
            );
        }
        if layer.buffer_size.is_some() {
            /*TODO: if layer type != point
            warn!(
                "Layer '{}': Clipping not supported for GDAL layers",
                layer.name
            );
            */
        }
    }
    fn retrieve_features<F>(
        &self,
        layer: &Layer,
        extent: &Extent,
        zoom: u8,
        grid: &Grid,
        mut read: F,
    ) -> u64
    where
        F: FnMut(&Feature),
    {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer_name = layer.table_name.as_ref().unwrap();
        debug!("retrieve_features layer: {}", layer_name);
        let ogr_layer = dataset.layer_by_name(layer_name).unwrap();

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

        // Spatial filter must be in layer SRS
        let transformation = self.bbox_transform.get(&layer.name).unwrap();
        if let Some(ref tr) = transformation {
            match transform_extent_tr(&bbox_extent, tr) {
                Ok(extent) => bbox_extent = extent,
                Err(e) => {
                    error!("Unable to transform {:?}: {}", bbox_extent, e);
                    return 0;
                }
            }
        };
        let bbox = Geometry::bbox(
            bbox_extent.minx,
            bbox_extent.miny,
            bbox_extent.maxx,
            bbox_extent.maxy,
        ).unwrap();
        ogr_layer.set_spatial_filter(&bbox);

        let transformation = self.geom_transform.get(&layer.name).unwrap();
        let fields_defn = ogr_layer.defn().fields().collect::<Vec<_>>();
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
