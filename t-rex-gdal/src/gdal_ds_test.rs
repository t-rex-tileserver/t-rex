//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use gdal_ds::GdalDatasource;
use datasource::DatasourceInput;
use core::feature::FeatureAttrValType;
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;


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
