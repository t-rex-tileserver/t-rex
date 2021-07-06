//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::feature::{Feature, FeatureAttr, FeatureAttrValType};
use crate::core::geom::*;
use crate::core::layer::Layer;
use postgres::types::{self, FromSql, Type};
use postgres::Row;
use std;

impl GeometryType {
    /// Convert returned geometry to core::geom::GeometryType based on GeometryType name
    pub fn from_geom_field(row: &Row, idx: &str, type_name: &str) -> Result<GeometryType, String> {
        let field = match type_name {
            "POINT" => row.try_get::<_, Point>(idx).map(|f| GeometryType::Point(f)),
            //"LINESTRING" =>
            //    row.try_get::<_, LineString>(idx).map(|f| GeometryType::LineString(f)),
            //"POLYGON" =>
            //    row.try_get::<_, Polygon>(idx).map(|f| GeometryType::Polygon(f)),
            "MULTIPOINT" => row
                .try_get::<_, MultiPoint>(idx)
                .map(|f| GeometryType::MultiPoint(f)),
            "LINESTRING" | "MULTILINESTRING" | "COMPOUNDCURVE" => row
                .try_get::<_, MultiLineString>(idx)
                .map(|f| GeometryType::MultiLineString(f)),
            "POLYGON" | "MULTIPOLYGON" | "CURVEPOLYGON" => row
                .try_get::<_, MultiPolygon>(idx)
                .map(|f| GeometryType::MultiPolygon(f)),
            "GEOMETRYCOLLECTION" => row
                .try_get::<_, GeometryCollection>(idx)
                .map(|f| GeometryType::GeometryCollection(f)),
            "GEOMETRY" => row.try_get::<_, Geometry>(idx).map(|geom| match geom {
                Geometry::Point(f) => GeometryType::Point(f),
                Geometry::LineString(f) => GeometryType::LineString(f),
                Geometry::Polygon(f) => GeometryType::Polygon(f),
                Geometry::MultiPoint(f) => GeometryType::MultiPoint(f),
                Geometry::MultiLineString(f) => GeometryType::MultiLineString(f),
                Geometry::MultiPolygon(f) => GeometryType::MultiPolygon(f),
                Geometry::GeometryCollection(f) => GeometryType::GeometryCollection(f),
            }),
            _ => {
                // PG geometry types:
                // CIRCULARSTRING, CIRCULARSTRINGM, COMPOUNDCURVE, COMPOUNDCURVEM, CURVEPOLYGON, CURVEPOLYGONM,
                // GEOMETRY, GEOMETRYCOLLECTION, GEOMETRYCOLLECTIONM, GEOMETRYM,
                // LINESTRING, LINESTRINGM, MULTICURVE, MULTICURVEM, MULTILINESTRING, MULTILINESTRINGM,
                // MULTIPOINT, MULTIPOINTM, MULTIPOLYGON, MULTIPOLYGONM, MULTISURFACE, MULTISURFACEM,
                // POINT, POINTM, POLYGON, POLYGONM,
                // POLYHEDRALSURFACE, POLYHEDRALSURFACEM, TIN, TINM, TRIANGLE, TRIANGLEM
                return Err(format!("Unknown geometry type {}", type_name));
            }
        };
        field.map_err(|e| e.to_string())
    }
}

impl<'a> FromSql<'a> for FeatureAttrValType {
    fn accepts(ty: &Type) -> bool {
        match ty {
            &types::Type::VARCHAR
            | &types::Type::VARCHAR_ARRAY
            | &types::Type::TEXT
            | &types::Type::CHAR_ARRAY
            | &types::Type::FLOAT4
            | &types::Type::FLOAT8
            | &types::Type::INT2
            | &types::Type::INT4
            | &types::Type::INT8
            | &types::Type::BOOL => true,
            _ => false,
        }
    }
    fn from_sql(ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        match ty {
            &types::Type::VARCHAR | &types::Type::TEXT | &types::Type::CHAR_ARRAY => {
                <String>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::String(v)))
            }
            &types::Type::VARCHAR_ARRAY => <Vec<String>>::from_sql(ty, raw)
                .and_then(|v| Ok(FeatureAttrValType::VarcharArray(v))),
            &types::Type::FLOAT4 => {
                <f32>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Float(v)))
            }
            &types::Type::FLOAT8 => {
                <f64>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Double(v)))
            }
            &types::Type::INT2 => {
                <i16>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Int(v as i64)))
            }
            &types::Type::INT4 => {
                <i32>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Int(v as i64)))
            }
            &types::Type::INT8 => {
                <i64>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Int(v)))
            }
            &types::Type::BOOL => {
                <bool>::from_sql(ty, raw).and_then(|v| Ok(FeatureAttrValType::Bool(v)))
            }
            _ => {
                let err: Box<dyn std::error::Error + Sync + Send> =
                    format!("cannot convert {} to FeatureAttrValType", ty).into();
                Err(err)
            }
        }
    }
}

pub(crate) struct FeatureRow<'a> {
    pub layer: &'a Layer,
    pub row: &'a Row,
}

impl<'a> Feature for FeatureRow<'a> {
    fn fid(&self) -> Option<u64> {
        self.layer.fid_field.as_ref().and_then(|fid| {
            let val = self.row.try_get::<_, FeatureAttrValType>(fid as &str);
            match val {
                Ok(FeatureAttrValType::Int(fid)) => Some(fid as u64),
                _ => None,
            }
        })
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        let mut attrs = Vec::new();
        for (i, col) in self.row.columns().into_iter().enumerate() {
            // Skip geometry_field and fid_field
            if col.name()
                != self
                    .layer
                    .geometry_field
                    .as_ref()
                    .unwrap_or(&"".to_string())
                && col.name() != self.layer.fid_field.as_ref().unwrap_or(&"".to_string())
            {
                let val = self.row.try_get::<_, Option<FeatureAttrValType>>(i);
                match val {
                    Ok(Some(v)) => {
                        let fattr = FeatureAttr {
                            key: col.name().to_string(),
                            value: v,
                        };
                        attrs.push(fattr);
                    }
                    Ok(None) => {
                        // Skip NULL values
                    }
                    Err(err) => {
                        warn!(
                            "Layer '{}' - skipping field '{}': {}",
                            self.layer.name,
                            col.name(),
                            err
                        );
                        //warn!("{:?}", self.row);
                    }
                }
            }
        }
        attrs
    }
    fn geometry(&self) -> Result<GeometryType, String> {
        let geom = GeometryType::from_geom_field(
            &self.row,
            &self
                .layer
                .geometry_field
                .as_ref()
                .expect("geometry_field undefined"),
            &self
                .layer
                .geometry_type
                .as_ref()
                .expect("geometry_type undefined"),
        );
        if let Err(ref err) = geom {
            error!("Layer '{}': {}", self.layer.name, err);
            error!("{:?}", self.row);
        }
        geom
    }
}
