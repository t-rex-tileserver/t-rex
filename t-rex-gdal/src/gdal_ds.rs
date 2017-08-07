//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::DatasourceInput;
use gdal;
use gdal::vector::{Dataset, Geometry, FieldValue};
use gdal_sys::ogr;
use core::feature::{Feature, FeatureAttr, FeatureAttrValType};
use core::geom::{self, GeometryType};
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use std::path::Path;


pub struct GdalDatasource {
    pub path: String,
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
        let geometry_type = unsafe { ogr::OGR_G_GetGeometryType(self.c_geometry()) };

        let ring = |n: usize| {
            let ring = unsafe { self._get_geometry(n) };
            return match ring.to_geo(srid) {
                       GeometryType::LineString(r) => r,
                       _ => panic!("Expected to get a LineString"),
                   };
        };

        match geometry_type {
            ogr::WKB_POINT => {
                let (x, y, _) = self.get_point(0); //TODO: ZM support?
                GeometryType::Point(geom::Point {
                                        x: x,
                                        y: y,
                                        srid: srid,
                                    })
            }
            ogr::WKB_MULTIPOINT => {
                let point_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as
                                  usize;
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
            ogr::WKB_LINESTRING => {
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
            ogr::WKB_MULTILINESTRING => {
                let string_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as
                                   usize;
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
            ogr::WKB_POLYGON => {
                let ring_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
                let rings = (0..ring_count).map(|n| ring(n)).collect();
                GeometryType::Polygon(geom::Polygon {
                                          rings: rings,
                                          srid: srid,
                                      })
            }
            ogr::WKB_MULTIPOLYGON => {
                let string_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as
                                   usize;
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
            ogr::WKB_GEOMETRYCOLLECTION => {
                let item_count = unsafe { ogr::OGR_G_GetGeometryCount(self.c_geometry()) } as usize;
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
    fn retrieve_features<F>(&self,
                            layer: &Layer,
                            _extent: &Extent,
                            _zoom: u8,
                            _grid: &Grid,
                            mut read: F)
        where F: FnMut(&Feature)
    {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let ogr_layer = dataset
            .layer_by_name(layer.table_name.as_ref().unwrap())
            .unwrap();
        let fields_defn = ogr_layer.defn().fields().collect::<Vec<_>>();
        for feature in ogr_layer.features() {
            let feat = VectorFeature {
                layer: layer,
                fields_defn: &fields_defn,
                feature: &feature,
            };
            read(&feat);
        }
    }
}



#[test]
fn test_gdal_api() {
    use std::path::Path;
    use gdal::vector::Dataset;

    let mut dataset = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let layer = dataset.layer_by_name("ne_10m_populated_places").unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap();
    let geometry = feature.geometry();
    assert_eq!(name_field.to_string(),
               Some("Colonia del Sacramento".to_string()));
    assert_eq!(geometry.wkt().unwrap(),
               "POINT (-6438719.62282072 -4093437.71441017)".to_string());
}

#[test]
fn test_gdal_retrieve_points() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    layer.fid_field = Some(String::from("SCALERANK"));
    layer.geometry_type = Some(String::from("POINT"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!("Ok(Point(Point { x: -6438719.622820721, y: -4093437.7144101723, srid: Some(3857) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        assert_eq!(3, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "SCALERANK");
        assert_eq!(feat.attributes()[1].key, "NAME");
        assert_eq!(feat.attributes()[2].key, "POP_MAX");
        if reccnt == 0 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Int(10));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("Colonia del Sacramento".to_string()));
            assert_eq!(feat.attributes()[2].value, FeatureAttrValType::Int(21714));
            assert_eq!(Some(10), feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(14644, reccnt);
}

#[test]
fn test_gdal_retrieve_multilines() {
    let mut layer = Layer::new("multilines");
    layer.table_name = Some(String::from("ne_10m_rivers_lake_centerlines"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    layer.geometry_type = Some(String::from("MULTILINE"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        if reccnt == 1398 {
            assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 524057.9769470511, y: 6406083.8740046825, srid: Some(3857) }, Point { x: 523360.4182889238, y: 6406083.8740046825, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "scalerank");
        assert_eq!(feat.attributes()[1].key, "name");
        if reccnt == 1398 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Double(8.0));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("Meuse".to_string()));
            assert_eq!(None, feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(1404, reccnt);
}

#[test]
fn test_gdal_retrieve_multipolys() {
    let mut layer = Layer::new("multipolys");
    layer.table_name = Some(String::from("ne_110m_admin_0_countries"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    layer.geometry_type = Some(String::from("MULTIPOLYGON"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        if reccnt == 97 {
            assert_eq!("Ok(MultiPolygon(MultiPolygonT { polygons: [PolygonT { rings: [LineStringT { points: [Point { x: 672711.8490145913, y: 6468481.737381289, srid: Some(3857) }, Point { x: 694939.8727280691, y: 6429360.233285289, srid: Some(3857) }, Point { x: 688658.0399394701, y: 6353928.606103209, srid: Some(3857) }, Point { x: 656535.5543245667, y: 6350309.278097487, srid: Some(3857) }, Point { x: 631632.5743412259, y: 6365185.930146941, srid: Some(3857) }, Point { x: 643695.764229205, y: 6461933.756740372, srid: Some(3857) }, Point { x: 672711.8490145913, y: 6468481.737381289, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "name");
        assert_eq!(feat.attributes()[1].key, "iso_a3");
        if reccnt == 97 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::String("Luxembourg".to_string()));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("LUX".to_string()));
            assert_eq!(None, feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(177, reccnt);
}
