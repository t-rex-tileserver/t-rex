//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use gdal::spatial_ref::{CoordTransform, SpatialRef};
use gdal::vector::{FieldValue, Geometry};
use gdal::Dataset;
use gdal_sys;
use std::path::Path;
use t_rex_core::core::feature::{Feature, FeatureAttr, FeatureAttrValType};
use t_rex_core::core::geom::{self, GeometryType};
use t_rex_core::core::layer::Layer;

fn ogr_type_name(ogr_type: OGRwkbGeometryType::Type) -> String {
    use std::ffi::CStr;
    let rv = unsafe { gdal_sys::OGRGeometryTypeToName(ogr_type) };
    //_string(rv)
    let c_str = unsafe { CStr::from_ptr(rv) };
    c_str.to_string_lossy().into_owned()
}

pub(crate) fn geom_type_name(ogr_type: OGRwkbGeometryType::Type) -> Option<String> {
    match ogr_type {
        OGRwkbGeometryType::wkbPoint | OGRwkbGeometryType::wkbMultiPoint => {
            Some("POINT".to_string())
        }
        OGRwkbGeometryType::wkbLineString | OGRwkbGeometryType::wkbMultiLineString => {
            Some("LINE".to_string())
        }
        OGRwkbGeometryType::wkbPolygon | OGRwkbGeometryType::wkbMultiPolygon => {
            Some("POLYGON".to_string())
        }
        _ => None,
    }
}

// OGRwkbGeometryType from GDAL 2.2 bindings
#[allow(non_snake_case, non_upper_case_globals, dead_code)]
pub mod OGRwkbGeometryType {
    /// List of well known binary geometry types.  These are used within the BLOBs
    /// but are also returned from OGRGeometry::getGeometryType() to identify the
    /// type of a geometry object.
    pub type Type = u32;
    /// < unknown type, non-standard
    pub const wkbUnknown: Type = 0;
    /// < 0-dimensional geometric object, standard WKB
    pub const wkbPoint: Type = 1;
    /// < 1-dimensional geometric object with linear
    /// interpolation between Points, standard WKB
    pub const wkbLineString: Type = 2;
    /// < planar 2-dimensional geometric object defined
    /// by 1 exterior boundary and 0 or more interior
    /// boundaries, standard WKB
    pub const wkbPolygon: Type = 3;
    /// < GeometryCollection of Points, standard WKB
    pub const wkbMultiPoint: Type = 4;
    /// < GeometryCollection of LineStrings, standard WKB
    pub const wkbMultiLineString: Type = 5;
    /// < GeometryCollection of Polygons, standard WKB
    pub const wkbMultiPolygon: Type = 6;
    /// < geometric object that is a collection of 1
    /// or more geometric objects, standard WKB
    pub const wkbGeometryCollection: Type = 7;
    /// < one or more circular arc segments connected end to end,
    /// ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCircularString: Type = 8;
    /// < sequence of contiguous curves, ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCompoundCurve: Type = 9;
    /// < planar surface, defined by 1 exterior boundary
    /// and zero or more interior boundaries, that are curves.
    /// ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCurvePolygon: Type = 10;
    /// < GeometryCollection of Curves, ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbMultiCurve: Type = 11;
    /// < GeometryCollection of Surfaces, ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbMultiSurface: Type = 12;
    /// < Curve (abstract type). ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurve: Type = 13;
    /// < Surface (abstract type). ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbSurface: Type = 14;
    /// < a contiguous collection of polygons, which share common boundary segments,
    /// ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbPolyhedralSurface: Type = 15;
    /// < a PolyhedralSurface consisting only of Triangle patches
    /// ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTIN: Type = 16;
    /// < a Triangle. ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTriangle: Type = 17;
    /// < non-standard, for pure attribute records
    pub const wkbNone: Type = 100;
    /// < non-standard, just for createGeometry()
    pub const wkbLinearRing: Type = 101;
    /// < wkbCircularString with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCircularStringZ: Type = 1008;
    /// < wkbCompoundCurve with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCompoundCurveZ: Type = 1009;
    /// < wkbCurvePolygon with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbCurvePolygonZ: Type = 1010;
    /// < wkbMultiCurve with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbMultiCurveZ: Type = 1011;
    /// < wkbMultiSurface with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.0
    pub const wkbMultiSurfaceZ: Type = 1012;
    /// < wkbCurve with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurveZ: Type = 1013;
    /// < wkbSurface with Z component. ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbSurfaceZ: Type = 1014;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbPolyhedralSurfaceZ: Type = 1015;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTINZ: Type = 1016;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTriangleZ: Type = 1017;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbPointM: Type = 2001;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbLineStringM: Type = 2002;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbPolygonM: Type = 2003;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiPointM: Type = 2004;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiLineStringM: Type = 2005;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiPolygonM: Type = 2006;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbGeometryCollectionM: Type = 2007;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCircularStringM: Type = 2008;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCompoundCurveM: Type = 2009;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurvePolygonM: Type = 2010;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiCurveM: Type = 2011;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiSurfaceM: Type = 2012;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurveM: Type = 2013;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbSurfaceM: Type = 2014;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbPolyhedralSurfaceM: Type = 2015;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTINM: Type = 2016;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTriangleM: Type = 2017;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbPointZM: Type = 3001;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbLineStringZM: Type = 3002;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbPolygonZM: Type = 3003;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiPointZM: Type = 3004;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiLineStringZM: Type = 3005;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiPolygonZM: Type = 3006;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbGeometryCollectionZM: Type = 3007;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCircularStringZM: Type = 3008;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCompoundCurveZM: Type = 3009;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurvePolygonZM: Type = 3010;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiCurveZM: Type = 3011;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbMultiSurfaceZM: Type = 3012;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbCurveZM: Type = 3013;
    /// < ISO SQL/MM Part 3. GDAL &gt;= 2.1
    pub const wkbSurfaceZM: Type = 3014;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbPolyhedralSurfaceZM: Type = 3015;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTINZM: Type = 3016;
    /// < ISO SQL/MM Part 3. Reserved in GDAL &gt;= 2.1 but not yet implemented
    pub const wkbTriangleZM: Type = 3017;
    /// < 2.5D extension as per 99-402
    pub const wkbPoint25D: Type = 2147483649;
    /// < 2.5D extension as per 99-402
    pub const wkbLineString25D: Type = 2147483650;
    /// < 2.5D extension as per 99-402
    pub const wkbPolygon25D: Type = 2147483651;
    /// < 2.5D extension as per 99-402
    pub const wkbMultiPoint25D: Type = 2147483652;
    /// < 2.5D extension as per 99-402
    pub const wkbMultiLineString25D: Type = 2147483653;
    /// < 2.5D extension as per 99-402
    pub const wkbMultiPolygon25D: Type = 2147483654;
    /// < 2.5D extension as per 99-402
    pub const wkbGeometryCollection25D: Type = 2147483655;
}

trait ToGeo {
    fn to_geo(&self, srid: Option<i32>) -> GeometryType;
}

impl ToGeo for Geometry {
    /// Convert OGR geomtry to t-rex EWKB geometry type (XY only)
    fn to_geo(&self, srid: Option<i32>) -> GeometryType {
        let geometry_type = self.geometry_type();

        let ring = |n: usize| {
            let ring = unsafe { self.get_unowned_geometry(n) };
            return match ring.to_geo(srid) {
                GeometryType::LineString(r) => r,
                _ => panic!("Expected to get a LineString"),
            };
        };

        match geometry_type {
            OGRwkbGeometryType::wkbPoint
            | OGRwkbGeometryType::wkbPoint25D
            | OGRwkbGeometryType::wkbPointM
            | OGRwkbGeometryType::wkbPointZM => {
                let (x, y, _) = self.get_point(0);
                GeometryType::Point(geom::Point {
                    x: x,
                    y: y,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbMultiPoint
            | OGRwkbGeometryType::wkbMultiPoint25D
            | OGRwkbGeometryType::wkbMultiPointM
            | OGRwkbGeometryType::wkbMultiPointZM => {
                let point_count = self.geometry_count();
                let coords = (0..point_count)
                    .map(
                        |n| match unsafe { self.get_unowned_geometry(n) }.to_geo(srid) {
                            GeometryType::Point(p) => p,
                            _ => panic!("Expected to get a Point"),
                        },
                    )
                    .collect();
                GeometryType::MultiPoint(geom::MultiPoint {
                    points: coords,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbLineString
            | OGRwkbGeometryType::wkbLineString25D
            | OGRwkbGeometryType::wkbLineStringM
            | OGRwkbGeometryType::wkbLineStringZM => {
                let coords = self
                    .get_point_vec()
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
            OGRwkbGeometryType::wkbMultiLineString
            | OGRwkbGeometryType::wkbMultiLineString25D
            | OGRwkbGeometryType::wkbMultiLineStringM
            | OGRwkbGeometryType::wkbMultiLineStringZM => {
                let string_count = self.geometry_count();
                let strings = (0..string_count)
                    .map(
                        |n| match unsafe { self.get_unowned_geometry(n) }.to_geo(srid) {
                            GeometryType::LineString(s) => s,
                            _ => panic!("Expected to get a LineString"),
                        },
                    )
                    .collect();
                GeometryType::MultiLineString(geom::MultiLineString {
                    lines: strings,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbPolygon
            | OGRwkbGeometryType::wkbPolygon25D
            | OGRwkbGeometryType::wkbPolygonM
            | OGRwkbGeometryType::wkbPolygonZM => {
                let ring_count = self.geometry_count();
                let rings = (0..ring_count).map(|n| ring(n)).collect();
                GeometryType::Polygon(geom::Polygon {
                    rings: rings,
                    srid: srid,
                })
            }
            OGRwkbGeometryType::wkbMultiPolygon
            | OGRwkbGeometryType::wkbMultiPolygon25D
            | OGRwkbGeometryType::wkbMultiPolygonM
            | OGRwkbGeometryType::wkbMultiPolygonZM => {
                let string_count = self.geometry_count();
                let strings = (0..string_count)
                    .map(
                        |n| match unsafe { self.get_unowned_geometry(n) }.to_geo(srid) {
                            GeometryType::Polygon(s) => s,
                            _ => panic!("Expected to get a Polygon"),
                        },
                    )
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
                    .map(|n| unsafe { self.get_unowned_geometry(n) }.to_geo(srid))
                    .collect();
                GeometryType::GeometryCollection(geom::GeometryCollection {
                                                     geometries: geometry_list,
                                                 })
            }
            */
            geom_type => panic!(
                "Unsupported geometry type {} ({})",
                geom_type,
                &ogr_type_name(geom_type)
            ),
        }
    }
}

pub fn ogr_layer_name(path: &str, id: isize) -> Result<String, gdal::errors::GdalError> {
    let dataset = Dataset::open(Path::new(path))?;
    let layer = dataset.layer(id)?;
    Ok(layer.name())
}

pub(crate) fn geom_spatialref<'d>(
    ogr_layer: &gdal::vector::Layer<'d>,
    field_name: Option<&String>,
) -> Option<SpatialRef> {
    if let Some(geom_field) = field_name {
        let geom_field = ogr_layer
            .defn()
            .geom_fields()
            .find(|f| &f.name() == geom_field);
        if let Some(field) = geom_field {
            field.spatial_ref().ok()
        } else {
            None
        }
    } else {
        ogr_layer.spatial_ref().ok()
    }
}

pub(crate) struct VectorFeature<'a> {
    pub layer: &'a Layer,
    pub fields_defn: &'a Vec<gdal::vector::Field<'a>>,
    pub grid_srid: i32,
    pub transform: Option<&'a CoordTransform>,
    pub feature: &'a gdal::vector::Feature<'a>,
}

impl<'a> Feature for VectorFeature<'a> {
    fn fid(&self) -> Option<u64> {
        self.layer.fid_field.as_ref().and_then(|fid| {
            let field_value = self.feature.field(&fid);
            match field_value {
                Ok(Some(FieldValue::IntegerValue(v))) => Some(v as u64),
                _ => None,
            }
        })
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        let mut attrs = Vec::new();
        for (_i, field) in self.fields_defn.into_iter().enumerate() {
            let field_value = self.feature.field(&field.name()); //TODO: get by index
            let val = match field_value {
                Ok(Some(FieldValue::StringValue(v))) => Some(FeatureAttrValType::String(v)),
                Ok(Some(FieldValue::IntegerValue(v))) => Some(FeatureAttrValType::Int(v as i64)),
                Ok(Some(FieldValue::Integer64Value(v))) => Some(FeatureAttrValType::Int(v)),
                Ok(Some(FieldValue::RealValue(v))) => Some(FeatureAttrValType::Double(v)),
                Ok(Some(FieldValue::IntegerListValue(_)))
                | Ok(Some(FieldValue::Integer64ListValue(_)))
                | Ok(Some(FieldValue::RealListValue(_)))
                | Ok(Some(FieldValue::StringListValue(_))) => {
                    // TODO: add support for list fields
                    warn!(
                        "Layer '{}' - skipping unsupported list field '{}'",
                        self.layer.name,
                        field.name()
                    );
                    None
                }
                Ok(None) => {
                    None // Skip NULL values
                }
                Err(err) => {
                    warn!(
                        "Layer '{}' - skipping field '{}': {:?}",
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
        let mut ogrgeom = ogrgeom.clone();
        if let Some(ref transform) = self.transform {
            ogrgeom.transform_inplace(transform).unwrap();
        };
        Ok(ogrgeom.to_geo(Some(self.grid_srid)))
    }
}
