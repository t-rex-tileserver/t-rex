//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::DatasourceInput;
use gdal;
use gdal::vector::{Dataset, Geometry, WkbType, FieldValue};
use core::feature::{Feature, FeatureAttr, FeatureAttrValType};
use core::geom::{self, GeometryType};
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use service::tileset::WORLD_EXTENT;
use std::path::Path;


pub struct GdalDatasource {
    pub path: String,
    // We don't store the Dataset, because we need mut access for getting layers
}

impl GdalDatasource {
    pub fn new(path: &str) -> GdalDatasource {
        GdalDatasource { path: path.to_string() }
    }
}


trait ToGeo {
    fn to_geo(&self, srid: Option<i32>) -> GeometryType;
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
            WkbType::WkbPoint => {
                let (x, y, _) = self.get_point(0);
                GeometryType::Point(geom::Point {
                                        x: x,
                                        y: y,
                                        srid: srid,
                                    })
            }
            WkbType::WkbMultipoint => {
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
            WkbType::WkbLinestring => {
                let coords = self.get_point_vec()
                    .iter()
                    .map(|&(x, y, _)| {
                             geom::Point {
                                 x: x,
                                 y: y,
                                 srid: srid,
                             }
                         })
                    .collect();
                GeometryType::LineString(geom::LineString {
                                             points: coords,
                                             srid: srid,
                                         })
            }
            WkbType::WkbMultilinestring => {
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
            WkbType::WkbPolygon => {
                let ring_count = self.geometry_count();
                let rings = (0..ring_count).map(|n| ring(n)).collect();
                GeometryType::Polygon(geom::Polygon {
                                          rings: rings,
                                          srid: srid,
                                      })
            }
            WkbType::WkbMultipolygon => {
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
            WkbType::WkbGeometrycollection => {
                let item_count = self.geometry_count();
                let geometry_list = (0..item_count)
                    .map(|n| unsafe { self._get_geometry(n) }.to_geo(srid))
                    .collect();
                GeometryType::GeometryCollection(geom::GeometryCollection {
                                                     geometries: geometry_list,
                                                 })
            }
            */
            _ => panic!("Unknown geometry type"),
        }
    }
}

struct VectorFeature<'a> {
    layer: &'a Layer,
    fields_defn: &'a Vec<gdal::vector::Field<'a>>,
    feature: &'a gdal::vector::Feature<'a>,
}


impl<'a> Feature for VectorFeature<'a> {
    fn fid(&self) -> Option<u64> {
        self.layer
            .fid_field
            .as_ref()
            .and_then(|fid| {
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
                    warn!("Layer '{}' - skipping field '{}': {}",
                          self.layer.name,
                          field.name(),
                          err);
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
        let ogrgeom = self.feature.geometry(); //FIXME: support for multiple geometry columns
        Ok(ogrgeom.to_geo(self.layer.srid))
    }
}

impl DatasourceInput for GdalDatasource {
    /// New instance with connected pool
    fn connected(&self) -> GdalDatasource {
        GdalDatasource { path: self.path.clone() }
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
    fn extent_from_wgs84(&self, _extent: &Extent, _dest_srid: i32) -> Option<Extent> {
        unimplemented!();
    }
    fn layer_extent(&self, _layer: &Layer) -> Option<Extent> {
        Some(WORLD_EXTENT.clone()) // TODO
    }
    fn prepare_queries(&mut self, _layer: &Layer, _grid_srid: i32) {
        // TODO: Prepare gdal::vector::Layer
    }
    fn retrieve_features<F>(&self,
                            layer: &Layer,
                            extent: &Extent,
                            _zoom: u8,
                            _grid: &Grid,
                            mut read: F)
        where F: FnMut(&Feature)
    {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap(); //TODO: Store gdal::vector::Layer
        let layer_name = layer.table_name.as_ref().unwrap();
        debug!("retrieve_features layer: {}", layer_name);
        let ogr_layer = dataset.layer_by_name(layer_name).unwrap();
        let bbox = if let Some(buffer_size) = layer.buffer_size {
            let buf = f64::from(buffer_size);
            Geometry::bbox(extent.minx - buf,
                           extent.miny - buf,
                           extent.maxx + buf,
                           extent.maxy + buf)
                    .unwrap()
        } else {
            Geometry::bbox(extent.minx, extent.miny, extent.maxx, extent.maxy).unwrap()
        };
        ogr_layer.set_spatial_filter(&bbox);
        let fields_defn = ogr_layer.defn().fields().collect::<Vec<_>>();
        let mut cnt = 0;
        let query_limit = layer.query_limit.unwrap_or(0);
        for feature in ogr_layer.features() {
            let feat = VectorFeature {
                layer: layer,
                fields_defn: &fields_defn,
                feature: &feature,
            };
            read(&feat);
            cnt += 1;
            if cnt == query_limit {
                info!("Feature count limited (query_limit={})", cnt);
                break;
            }
        }
        debug!("Feature count: {}", cnt);
    }
}
