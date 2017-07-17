extern crate gdal;


#[test]
fn test_gdal() {
    use std::path::Path;
    use gdal::vector::Dataset;

    let mut dataset = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let layer = dataset.layer(0).unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap();
    let geometry = feature.geometry();
    assert_eq!(name_field.to_string(), Some("Colonia del Sacramento".to_string()));
    assert_eq!(geometry.wkt().unwrap(), "POINT (-6438719.62282072 -4093437.71441017)".to_string());
}
