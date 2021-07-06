//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::geom::{GeometryType, Point};

/// Supported feature attribute value types
#[derive(Clone, PartialEq, Debug)]
pub enum FeatureAttrValType {
    String(String), //TODO: use ref
    Float(f32),
    Double(f64),
    Int(i64),
    UInt(u64),
    SInt(i64),
    Bool(bool),
    VarcharArray(Vec<String>),
}

pub trait Feature {
    fn fid(&self) -> Option<u64>;
    fn attributes(&self) -> Vec<FeatureAttr>; //TODO: return tuples
    fn geometry(&self) -> Result<GeometryType, String>;
}

#[derive(Clone, Debug)]
pub struct FeatureAttr {
    pub key: String,
    pub value: FeatureAttrValType,
}

/// Basic Feature implementation
// Only used for encoding tests
pub struct FeatureStruct {
    pub fid: Option<u64>,
    pub attributes: Vec<FeatureAttr>,
    pub geometry: GeometryType,
}

impl Feature for FeatureStruct {
    fn fid(&self) -> Option<u64> {
        self.fid
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        self.attributes.clone()
    }
    fn geometry(&self) -> Result<GeometryType, String> {
        // Return dummy geometry because of missing Clone impl
        Ok(GeometryType::Point(Point::new(0.0, 0.0, None)))
    }
}
