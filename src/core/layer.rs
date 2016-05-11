#[derive(Default)]
pub struct Layer {
    pub name: String,
    pub query:  Option<String>,
    pub geometry_field: Option<String>,
    pub geometry_type: Option<String>,
    pub fid_field: Option<String>,
    pub query_limit: Option<u32>,
}

impl Layer {
    pub fn new(name: &str) -> Layer {
        Layer { name: String::from(name), ..Default::default() }
    }
}
