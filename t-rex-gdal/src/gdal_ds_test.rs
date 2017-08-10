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
use std::path::Path;
use gdal::vector::Dataset;


#[test]
fn test_gdal_api() {

    let mut dataset = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let layer = dataset.layer_by_name("ne_10m_populated_places").unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap();
    let geometry = feature.geometry();
    assert_eq!(name_field.to_string(),
               Some("Colonia del Sacramento".to_string()));
    #[cfg(not(target_os = "macos"))]
    assert_eq!(geometry.wkt().unwrap(),
               "POINT (-6438719.62282072 -4093437.71441017)".to_string());
    #[cfg(target_os = "macos")]
    assert_eq!(geometry.wkt().unwrap(),
               "POINT (-6438719.622820721007884 -4093437.714410172309726)".to_string());
}

#[test]
fn test_detect_layers() {
    let ds = GdalDatasource::new("natural_earth.gpkg");
    let layers = ds.detect_layers(true);
    println!("{:?}", layers);
    assert_eq!(layers.len(), 3);
    assert_eq!(format!("{:?}", layers[0]), r#"Layer { name: "ne_10m_populated_places", geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_10m_populated_places"), query_limit: None, query: [], simplify: None, buffer_size: None, style: None }"#);
    assert_eq!(format!("{:?}", layers[1]), r#"Layer { name: "ne_10m_rivers_lake_centerlines", geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_10m_rivers_lake_centerlines"), query_limit: None, query: [], simplify: None, buffer_size: None, style: None }"#);
    assert_eq!(format!("{:?}", layers[2]), r#"Layer { name: "ne_110m_admin_0_countries", geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_110m_admin_0_countries"), query_limit: None, query: [], simplify: None, buffer_size: None, style: None }"#);
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
            assert_eq!("Ok(Point(Point { x: 831219.9062494118, y: 5928485.165733484, srid: Some(3857) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        assert_eq!(3, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "SCALERANK");
        assert_eq!(feat.attributes()[1].key, "NAME");
        assert_eq!(feat.attributes()[2].key, "POP_MAX");
        if reccnt == 0 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Int(4));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("Bern".to_string()));
            assert_eq!(feat.attributes()[2].value, FeatureAttrValType::Int(275329));
            assert_eq!(feat.fid(), Some(4));
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 2);
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

    let mut gdal_ds = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let gdal_layer = gdal_ds
        .layer_by_name(layer.table_name.as_ref().unwrap())
        .unwrap();
    assert_eq!(gdal_layer.features().count(), 1404);

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;

    // without buffer
    ds.retrieve_features(&layer, &extent, 10, &grid, |_| { reccnt += 1; });
    assert_eq!(reccnt, 0);

    // with buffer
    layer.buffer_size = Some(100000);
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        if reccnt == 1 {
            assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 682757.1012729447, y: 5813200.024936108, srid: Some(3857) }, Point { x: 683572.4295746532, y: 5814895.307100639, srid: Some(3857) }, Point { x: 684405.8762830653, y: 5815700.51643066, srid: Some(3857) }, Point { x: 686063.7104965394, y: 5817684.394041292, srid: Some(3857) }, Point { x: 687404.4725926834, y: 5820284.406662052, srid: Some(3857) }, Point { x: 688545.9322150754, y: 5823494.54626182, srid: Some(3857) }, Point { x: 691689.4757783283, y: 5831092.159555616, srid: Some(3857) }, Point { x: 692287.3831995813, y: 5833289.410580246, srid: Some(3857) }, Point { x: 694633.7168678325, y: 5836484.600127116, srid: Some(3857) }, Point { x: 697804.4380411424, y: 5840698.509721698, srid: Some(3857) }, Point { x: 701428.1193820703, y: 5843830.710570353, srid: Some(3857) }, Point { x: 704852.4982492463, y: 5844947.27781533, srid: Some(3857) }, Point { x: 708512.4164035813, y: 5845118.059404142, srid: Some(3857) }, Point { x: 712136.0977445092, y: 5846050.848060609, srid: Some(3857) }, Point { x: 721612.0244510319, y: 5852583.137171743, srid: Some(3857) }, Point { x: 728741.6174893066, y: 5853983.544356565, srid: Some(3857) }, Point { x: 736161.1050348545, y: 5853746.840167029, srid: Some(3857) }, Point { x: 752766.6247796519, y: 5849447.822084563, srid: Some(3857) }, Point { x: 752676.0327461276, y: 5849717.2707096655, srid: Some(3857) }, Point { x: 761236.9799140673, y: 5847509.349466373, srid: Some(3857) }, Point { x: 763248.1230582818, y: 5846142.818490322, srid: Some(3857) }, Point { x: 764335.2274605598, y: 5845131.196586606, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "scalerank");
        assert_eq!(feat.attributes()[1].key, "name");
        if reccnt == 1 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Double(6.0));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("Rhne".to_string()));
            assert_eq!(None, feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 5);
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
        if reccnt == 0 {
            assert_eq!("Ok(MultiPolygon(MultiPolygonT { polygons: [PolygonT { rings: [LineStringT { points: [Point { x: 1068024.3649477786, y: 6028202.019",
                       &format!("{:?}", feat.geometry())[0..130]);
        }
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "name");
        assert_eq!(feat.attributes()[1].key, "iso_a3");
        if reccnt == 0 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::String("Switzerland".to_string()));
            assert_eq!(feat.attributes()[1].value,
                       FeatureAttrValType::String("CHE".to_string()));
            assert_eq!(None, feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 1);
}
