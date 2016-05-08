pub struct Layer {
    pub name: String,
    pub query: String,
}

impl Layer {
    pub fn geometry_field(&self) -> String { String::from("geometry") } //TODO
    pub fn geometry_type(&self) -> String { String::from("POINT") } //TODO
}
