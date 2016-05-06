use core::geom::GeometryType;

/// Supported feature attribute value types
pub enum FeatureAttrValType {
    String(String),
    Float(f32),
    Double(f64),
    Int(i64),
    UInt(u64),
    SInt(i64),
    Bool(bool)
}

pub struct FeatureAttr {
    pub key: String,
    pub value: FeatureAttrValType
}

pub struct Feature {
    pub fid: Option<u64>,
    pub attributes: Vec<FeatureAttr>,
    pub geometry: GeometryType,
}
