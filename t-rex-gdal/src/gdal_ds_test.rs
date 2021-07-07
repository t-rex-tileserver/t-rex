//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::gdal_ds::GdalDatasource;
use gdal::Dataset;
use std::path::Path;
use t_rex_core::core::feature::FeatureAttrValType;
use t_rex_core::core::layer::Layer;
use t_rex_core::datasource::DatasourceType;
use tile_grid::Extent;
use tile_grid::Grid;

fn gdal_version() -> i32 {
    gdal::version::version_info("VERSION_NUM")
        .parse::<i32>()
        .unwrap()
}

#[test]
fn test_gdal_api() {
    let dataset = Dataset::open(Path::new("../data/natural_earth.gpkg")).unwrap();
    let mut layer = dataset.layer_by_name("ne_10m_populated_places").unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap().unwrap();
    let geometry = feature.geometry();
    assert_eq!(
        name_field.into_string(),
        Some("Colonia del Sacramento".to_string())
    );
    if gdal_version() >= 2000000 {
        assert_eq!(
            geometry.wkt().unwrap(),
            "POINT (-6438719.62282072 -4093437.71441017)".to_string()
        );
    } else {
        // GDAL 1.11 on MacOS
        assert_eq!(
            geometry.wkt().unwrap(),
            "POINT (-6438719.622820721007884 -4093437.714410172309726)".to_string()
        );
    };
}

#[test]
fn test_detect_layers() {
    let ds = GdalDatasource::new("../data/natural_earth.gpkg");
    let layers = ds.detect_layers(true);
    println!("{:?}", layers);
    assert_eq!(layers.len(), 3);
    assert_eq!(
        format!("{:?}", layers[0]),
        r#"Layer { name: "ne_10m_populated_places", datasource: None, geometry_field: Some("geom"), geometry_type: Some("POINT"), srid: Some(3857), no_transform: false, fid_field: None, table_name: Some("ne_10m_populated_places"), query_limit: None, query: [], minzoom: None, maxzoom: None, tile_size: 4096, simplify: false, tolerance: "", buffer_size: None, make_valid: false, shift_longitude: false, style: None }"#
    );
    assert_eq!(
        format!("{:?}", layers[1]),
        r#"Layer { name: "ne_10m_rivers_lake_centerlines", datasource: None, geometry_field: Some("geom"), geometry_type: Some("LINE"), srid: Some(3857), no_transform: false, fid_field: None, table_name: Some("ne_10m_rivers_lake_centerlines"), query_limit: None, query: [], minzoom: None, maxzoom: None, tile_size: 4096, simplify: false, tolerance: "", buffer_size: None, make_valid: false, shift_longitude: false, style: None }"#
    );
    assert_eq!(
        format!("{:?}", layers[2]),
        r#"Layer { name: "ne_110m_admin_0_countries", datasource: None, geometry_field: Some("geom"), geometry_type: Some("POLYGON"), srid: Some(3857), no_transform: false, fid_field: None, table_name: Some("ne_110m_admin_0_countries"), query_limit: None, query: [], minzoom: None, maxzoom: None, tile_size: 4096, simplify: false, tolerance: "", buffer_size: None, make_valid: false, shift_longitude: false, style: None }"#
    );
}

#[test]
fn test_gdal_retrieve_points() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    layer.fid_field = Some(String::from("SCALERANK"));
    //layer.geometry_type = Some(String::from("POINT"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let mut ds = GdalDatasource::new("../data/natural_earth.gpkg");
    ds.prepare_queries("ts", &layer, grid.srid);
    let mut reccnt = 0;
    ds.retrieve_features("ts", &layer, &extent, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!(
                "Ok(Point(Point { x: 831219.91, y: 5928485.17, srid: Some(3857) }))",
                &*format!("{:.2?}", feat.geometry())
            );
        }
        assert_eq!(3, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "SCALERANK");
        assert_eq!(feat.attributes()[1].key, "NAME");
        assert_eq!(feat.attributes()[2].key, "POP_MAX");
        if reccnt == 0 {
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Int(4));
            assert_eq!(
                feat.attributes()[1].value,
                FeatureAttrValType::String("Bern".to_string())
            );
            assert_eq!(feat.attributes()[2].value, FeatureAttrValType::Int(275329));
            assert_eq!(feat.fid(), Some(4));
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 1);
}

#[test]
fn test_coord_transformation() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    let grid = Grid::wgs84();
    let mut ds = GdalDatasource::new("../data/natural_earth.gpkg");
    ds.prepare_queries("ts", &layer, grid.srid);

    let extent_wgs84 = Extent {
        minx: 7.3828,
        miny: 46.8000,
        maxx: 7.7343,
        maxy: 47.0401,
    };
    let extent_3857 = if gdal_version() < 3000000 {
        Extent {
            minx: 821849.5366285803,
            miny: 5909489.863677091,
            maxx: 860978.3376424159,
            maxy: 5948621.871058013,
        }
    } else {
        Extent {
            minx: 821849.5366285802,
            miny: 5909489.863677087,
            maxx: 860978.3376424158,
            maxy: 5948621.871058013,
        }
    };
    assert_eq!(
        ds.reproject_extent(&extent_wgs84, 3857, None),
        Some(extent_3857.clone())
    );

    // Invalid input extent doesn't panic
    let result = ds.reproject_extent(&extent_3857, 3857, None);
    assert!(result.is_none());

    let mut reccnt = 0;
    let point_bern = if gdal_version() < 3000000 {
        "Ok(Point(Point { x: 7.466975462482421, y: 46.916682758667704, srid: Some(4326) }))"
    } else {
        "Ok(Point(Point { x: 7.466975462482424, y: 46.91668275866772, srid: Some(4326) }))"
    };
    ds.retrieve_features("ts", &layer, &extent_wgs84, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!(point_bern, &*format!("{:?}", feat.geometry()));
            assert_eq!(
                feat.attributes()[1].value,
                FeatureAttrValType::String("Bern".to_string())
            );
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 1);
}

#[test]
fn test_gdal_retrieve_multilines() {
    let mut layer = Layer::new("multilines");
    layer.table_name = Some(String::from("ne_10m_rivers_lake_centerlines"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    //layer.geometry_type = Some(String::from("MULTILINE"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let gdal_ds = Dataset::open(Path::new("../data/natural_earth.gpkg")).unwrap();
    let mut gdal_layer = gdal_ds
        .layer_by_name(layer.table_name.as_ref().unwrap())
        .unwrap();
    assert_eq!(gdal_layer.features().count(), 1404);

    let mut ds = GdalDatasource::new("../data/natural_earth.gpkg");
    ds.prepare_queries("ts", &layer, grid.srid);
    let mut reccnt = 0;

    // without buffer
    ds.retrieve_features("ds", &layer, &extent, 10, &grid, |_| {
        reccnt += 1;
    });
    assert_eq!(reccnt, 0);

    // with buffer
    layer.buffer_size = Some(600);

    ds.retrieve_features("ds", &layer, &extent, 22, &grid, |_| {
        reccnt += 1;
    });
    assert_eq!(reccnt, 0);

    let mut reccnt = 0;
    ds.retrieve_features("ds", &layer, &extent, 10, &grid, |feat| {
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "scalerank");
        assert_eq!(feat.attributes()[1].key, "name");
        if reccnt == 1 {
            assert_eq!(
                feat.attributes()[1].value,
                FeatureAttrValType::String("Rhne".to_string())
            );
            assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Double(6.0));
            assert_eq!(None, feat.fid());
            if gdal_version() < 2020000 {
                assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 682757.10, y: 5813200.02, srid: Some(3857) }, Point { x: 683572.43, y: 5814895.31, srid: Some(3857) }, Point { x: 684405.88, y: 5815700.52, srid: Some(3857) }, Point { x: 686063.71, y: 5817684.39, srid: Some(3857) }, Point { x: 687404.47, y: 5820284.41, srid: Some(3857) }, Point { x: 688545.93, y: 5823494.55, srid: Some(3857) }, Point { x: 691689.48, y: 5831092.16, srid: Some(3857) }, Point { x: 692287.38, y: 5833289.41, srid: Some(3857) }, Point { x: 694633.72, y: 5836484.60, srid: Some(3857) }, Point { x: 697804.44, y: 5840698.51, srid: Some(3857) }, Point { x: 701428.12, y: 5843830.71, srid: Some(3857) }, Point { x: 704852.50, y: 5844947.28, srid: Some(3857) }, Point { x: 708512.42, y: 5845118.06, srid: Some(3857) }, Point { x: 712136.10, y: 5846050.85, srid: Some(3857) }, Point { x: 721612.02, y: 5852583.14, srid: Some(3857) }, Point { x: 728741.62, y: 5853983.54, srid: Some(3857) }, Point { x: 736161.11, y: 5853746.84, srid: Some(3857) }, Point { x: 752766.62, y: 5849447.82, srid: Some(3857) }, Point { x: 752676.03, y: 5849717.27, srid: Some(3857) }, Point { x: 761236.98, y: 5847509.35, srid: Some(3857) }, Point { x: 763248.12, y: 5846142.82, srid: Some(3857) }, Point { x: 764335.23, y: 5845131.20, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                           &*format!("{:.2?}", feat.geometry()));
            } else {
                assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 932700.52, y: 5873592.93, srid: Some(3857) }, Point { x: 931649.65, y: 5871280.18, srid: Some(3857) }, Point { x: 930263.60, y: 5869969.24, srid: Some(3857) }, Point { x: 929312.38, y: 5867874.77, srid: Some(3857) }, Point { x: 926784.86, y: 5865240.92, srid: Some(3857) }, Point { x: 923134.00, y: 5862239.30, srid: Some(3857) }, Point { x: 919628.09, y: 5859034.75, srid: Some(3857) }, Point { x: 918441.34, y: 5857969.02, srid: Some(3857) }, Point { x: 902406.55, y: 5843633.68, srid: Some(3857) }, Point { x: 894814.93, y: 5838630.69, srid: Some(3857) }, Point { x: 893791.24, y: 5836464.91, srid: Some(3857) }, Point { x: 891390.55, y: 5834693.31, srid: Some(3857) }, Point { x: 886453.29, y: 5832528.51, srid: Some(3857) }, Point { x: 882132.05, y: 5831603.71, srid: Some(3857) }, Point { x: 868135.58, y: 5832528.51, srid: Some(3857) }, Point { x: 849002.54, y: 5833617.41, srid: Some(3857) }, Point { x: 846502.20, y: 5832941.75, srid: Some(3857) }, Point { x: 842452.74, y: 5829997.02, srid: Some(3857) }, Point { x: 832279.25, y: 5827020.49, srid: Some(3857) }, Point { x: 827061.15, y: 5822328.28, srid: Some(3857) }, Point { x: 822794.27, y: 5821135.96, srid: Some(3857) }, Point { x: 819487.66, y: 5818450.57, srid: Some(3857) }, Point { x: 800979.71, y: 5809444.07, srid: Some(3857) }, Point { x: 790597.86, y: 5801544.77, srid: Some(3857) }, Point { x: 787227.83, y: 5800923.84, srid: Some(3857) }, Point { x: 786249.44, y: 5801845.44, srid: Some(3857) }, Point { x: 784990.21, y: 5803675.84, srid: Some(3857) }, Point { x: 783903.11, y: 5805663.56, srid: Some(3857) }, Point { x: 783441.09, y: 5807010.75, srid: Some(3857) }, Point { x: 783341.44, y: 5810346.93, srid: Some(3857) }, Point { x: 782951.89, y: 5812467.03, srid: Some(3857) }, Point { x: 782118.44, y: 5814050.90, srid: Some(3857) }, Point { x: 779618.10, y: 5817409.37, srid: Some(3857) }, Point { x: 772551.92, y: 5829760.96, srid: Some(3857) }, Point { x: 772035.55, y: 5831229.88, srid: Some(3857) }, Point { x: 771546.35, y: 5834076.62, srid: Some(3857) }, Point { x: 770350.54, y: 5835027.92, srid: Some(3857) }, Point { x: 768855.77, y: 5835342.86, srid: Some(3857) }, Point { x: 767469.71, y: 5836287.74, srid: Some(3857) }, Point { x: 767052.99, y: 5837488.67, srid: Some(3857) }, Point { x: 767188.88, y: 5838689.76, srid: Some(3857) }, Point { x: 767460.65, y: 5839772.85, srid: Some(3857) }, Point { x: 767469.71, y: 5840600.03, srid: Some(3857) }, Point { x: 766763.09, y: 5841978.83, srid: Some(3857) }, Point { x: 765105.26, y: 5843935.79, srid: Some(3857) }, Point { x: 764362.41, y: 5845104.92, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 682757.10, y: 5813200.02, srid: Some(3857) }, Point { x: 681534.11, y: 5813304.74, srid: Some(3857) }, Point { x: 680003.10, y: 5813684.36, srid: Some(3857) }, Point { x: 678888.82, y: 5814194.90, srid: Some(3857) }, Point { x: 678318.09, y: 5814378.18, srid: Some(3857) }, Point { x: 678182.20, y: 5814541.82, srid: Some(3857) }, Point { x: 677049.80, y: 5814234.17, srid: Some(3857) }, Point { x: 675935.52, y: 5813854.54, srid: Some(3857) }, Point { x: 674794.06, y: 5813671.27, srid: Some(3857) }, Point { x: 673571.07, y: 5813200.02, srid: Some(3857) }, Point { x: 670146.69, y: 5809797.35, srid: Some(3857) }, Point { x: 668108.37, y: 5808534.77, srid: Some(3857) }, Point { x: 664321.62, y: 5802682.15, srid: Some(3857) }, Point { x: 662863.09, y: 5800923.84, srid: Some(3857) }, Point { x: 661775.99, y: 5800747.38, srid: Some(3857) }, Point { x: 659384.36, y: 5801139.53, srid: Some(3857) }, Point { x: 658315.37, y: 5800923.84, srid: Some(3857) }, Point { x: 657400.39, y: 5800021.95, srid: Some(3857) }, Point { x: 656005.27, y: 5797336.43, srid: Some(3857) }, Point { x: 655253.36, y: 5796278.12, srid: Some(3857) }, Point { x: 653622.70, y: 5795317.92, srid: Some(3857) }, Point { x: 652743.96, y: 5795944.98, srid: Some(3857) }, Point { x: 651992.05, y: 5797199.23, srid: Some(3857) }, Point { x: 650769.05, y: 5798159.64, srid: Some(3857) }, Point { x: 648458.96, y: 5795631.44, srid: Some(3857) }, Point { x: 648350.25, y: 5788403.61, srid: Some(3857) }, Point { x: 650008.08, y: 5775633.84, srid: Some(3857) }, Point { x: 650171.15, y: 5769607.52, srid: Some(3857) }, Point { x: 649210.87, y: 5764053.90, srid: Some(3857) }, Point { x: 645605.31, y: 5753444.62, srid: Some(3857) }, Point { x: 644762.80, y: 5750129.78, srid: Some(3857) }, Point { x: 644409.49, y: 5747868.60, srid: Some(3857) }, Point { x: 642914.72, y: 5747634.72, srid: Some(3857) }, Point { x: 641012.29, y: 5747634.72, srid: Some(3857) }, Point { x: 638511.95, y: 5747166.97, srid: Some(3857) }, Point { x: 636799.76, y: 5744750.68, srid: Some(3857) }, Point { x: 635413.70, y: 5741510.49, srid: Some(3857) }, Point { x: 635486.18, y: 5737836.68, srid: Some(3857) }, Point { x: 636011.61, y: 5734508.19, srid: Some(3857) }, Point { x: 635957.26, y: 5732426.08, srid: Some(3857) }, Point { x: 633864.58, y: 5730344.46, srid: Some(3857) }, Point { x: 630132.19, y: 5725560.52, srid: Some(3857) }, Point { x: 627939.86, y: 5723765.58, srid: Some(3857) }, Point { x: 625602.59, y: 5723441.62, srid: Some(3857) }, Point { x: 623446.50, y: 5725327.22, srid: Some(3857) }, Point { x: 619596.34, y: 5732620.65, srid: Some(3857) }, Point { x: 617494.60, y: 5735520.23, srid: Some(3857) }, Point { x: 609776.16, y: 5742627.22, srid: Some(3857) }, Point { x: 606714.15, y: 5747238.43, srid: Some(3857) }, Point { x: 604802.66, y: 5753321.10, srid: Some(3857) }, Point { x: 604947.60, y: 5753464.12, srid: Some(3857) }, Point { x: 605300.91, y: 5753600.64, srid: Some(3857) }, Point { x: 605463.98, y: 5753737.16, srid: Some(3857) }, Point { x: 605020.08, y: 5754621.36, srid: Some(3857) }, Point { x: 604485.58, y: 5755427.61, srid: Some(3857) }, Point { x: 599448.67, y: 5760162.57, srid: Some(3857) }, Point { x: 597682.12, y: 5761131.99, srid: Some(3857) }, Point { x: 596893.97, y: 5761014.88, srid: Some(3857) }, Point { x: 596350.42, y: 5760487.87, srid: Some(3857) }, Point { x: 595236.14, y: 5760162.57, srid: Some(3857) }, Point { x: 593723.25, y: 5759219.28, srid: Some(3857) }, Point { x: 592083.53, y: 5756988.31, srid: Some(3857) }, Point { x: 586974.14, y: 5748329.89, srid: Some(3857) }, Point { x: 585243.84, y: 5746718.74, srid: Some(3857) }, Point { x: 583223.63, y: 5746069.17, srid: Some(3857) }, Point { x: 581076.60, y: 5746575.83, srid: Some(3857) }, Point { x: 577869.64, y: 5748823.69, srid: Some(3857) }, Point { x: 575179.06, y: 5749798.37, srid: Some(3857) }, Point { x: 571491.97, y: 5751982.04, srid: Some(3857) }, Point { x: 569779.78, y: 5752606.04, srid: Some(3857) }, Point { x: 568004.17, y: 5752541.04, srid: Some(3857) }, Point { x: 564099.66, y: 5751657.05, srid: Some(3857) }, Point { x: 562197.22, y: 5751514.06, srid: Some(3857) }, Point { x: 555728.95, y: 5752606.04, srid: Some(3857) }, Point { x: 553880.87, y: 5752281.03, srid: Some(3857) }, Point { x: 550692.03, y: 5750851.14, srid: Some(3857) }, Point { x: 547349.19, y: 5750168.77, srid: Some(3857) }, Point { x: 542946.42, y: 5748225.93, srid: Some(3857) }, Point { x: 541596.59, y: 5747147.48, srid: Some(3857) }, Point { x: 541179.87, y: 5744887.06, srid: Some(3857) }, Point { x: 541134.58, y: 5742003.91, srid: Some(3857) }, Point { x: 540563.85, y: 5740367.94, srid: Some(3857) }, Point { x: 538570.82, y: 5741796.15, srid: Some(3857) }, Point { x: 537737.37, y: 5740757.43, srid: Some(3857) }, Point { x: 537048.87, y: 5739524.11, srid: Some(3857) }, Point { x: 539195.91, y: 5735169.89, srid: Some(3857) }, Point { x: 538842.60, y: 5729722.01, srid: Some(3857) }, Point { x: 536912.99, y: 5724141.39, srid: Some(3857) }, Point { x: 534313.00, y: 5719347.82, srid: Some(3857) }, Point { x: 534602.89, y: 5715359.46, srid: Some(3857) }, Point { x: 538706.71, y: 5712175.23, srid: Some(3857) }, Point { x: 542131.09, y: 5708474.67, srid: Some(3857) }, Point { x: 540427.96, y: 5702965.49, srid: Some(3857) }, Point { x: 532818.23, y: 5696309.85, srid: Some(3857) }, Point { x: 529883.04, y: 5691763.60, srid: Some(3857) }, Point { x: 529366.67, y: 5685038.88, srid: Some(3857) }, Point { x: 531377.81, y: 5677842.17, srid: Some(3857) }, Point { x: 531649.59, y: 5674761.48, srid: Some(3857) }, Point { x: 532492.10, y: 5672815.64, srid: Some(3857) }, Point { x: 534222.40, y: 5670999.05, srid: Some(3857) }, Point { x: 535617.52, y: 5668770.69, srid: Some(3857) }, Point { x: 535526.93, y: 5665570.80, srid: Some(3857) }, Point { x: 536215.43, y: 5665570.80, srid: Some(3857) }, Point { x: 535726.23, y: 5662577.98, srid: Some(3857) }, Point { x: 535617.52, y: 5657598.58, srid: Some(3857) }, Point { x: 535907.41, y: 5652647.65, srid: Some(3857) }, Point { x: 537375.01, y: 5646414.65, srid: Some(3857) }, Point { x: 536831.45, y: 5642445.77, srid: Some(3857) }, Point { x: 536867.69, y: 5639043.43, srid: Some(3857) }, Point { x: 539313.68, y: 5637439.00, srid: Some(3857) }, Point { x: 538906.01, y: 5635437.07, srid: Some(3857) }, Point { x: 539440.50, y: 5634243.83, srid: Some(3857) }, Point { x: 540283.01, y: 5633255.99, srid: Some(3857) }, Point { x: 540835.62, y: 5631857.81, srid: Some(3857) }, Point { x: 540853.74, y: 5629998.18, srid: Some(3857) }, Point { x: 540183.36, y: 5626760.78, srid: Some(3857) }, Point { x: 540011.23, y: 5624915.02, srid: Some(3857) }, Point { x: 540726.91, y: 5621884.43, srid: Some(3857) }, Point { x: 543897.63, y: 5616287.25, srid: Some(3857) }, Point { x: 544640.49, y: 5613579.57, srid: Some(3857) }, Point { x: 544196.59, y: 5610859.90, srid: Some(3857) }, Point { x: 541968.02, y: 5603409.00, srid: Some(3857) }, Point { x: 541216.11, y: 5601874.81, srid: Some(3857) }, Point { x: 540799.38, y: 5601254.82, srid: Some(3857) }, Point { x: 538326.22, y: 5598577.19, srid: Some(3857) }, Point { x: 537737.37, y: 5598078.82, srid: Some(3857) }, Point { x: 537483.72, y: 5595900.36, srid: Some(3857) }, Point { x: 536813.34, y: 5594916.71, srid: Some(3857) }, Point { x: 535853.06, y: 5594316.35, srid: Some(3857) }, Point { x: 534693.48, y: 5593307.33, srid: Some(3857) }, Point { x: 533026.59, y: 5591289.63, srid: Some(3857) }, Point { x: 531839.83, y: 5589374.50, srid: Some(3857) }, Point { x: 531133.21, y: 5586853.54, srid: Some(3857) }, Point { x: 530888.62, y: 5582987.30, srid: Some(3857) }, Point { x: 531323.46, y: 5580914.50, srid: Some(3857) }, Point { x: 533352.72, y: 5576413.37, srid: Some(3857) }, Point { x: 533932.51, y: 5573845.03, srid: Some(3857) }, Point { x: 533162.48, y: 5567914.51, srid: Some(3857) }, Point { x: 530616.84, y: 5563687.15, srid: Some(3857) }, Point { x: 527654.48, y: 5559748.06, srid: Some(3857) }, Point { x: 525579.92, y: 5554615.17, srid: Some(3857) }, Point { x: 523577.84, y: 5539202.13, srid: Some(3857) }, Point { x: 523161.12, y: 5537767.62, srid: Some(3857) }, Point { x: 520642.66, y: 5533351.25, srid: Some(3857) }, Point { x: 520099.11, y: 5532000.12, srid: Some(3857) }, Point { x: 519899.80, y: 5530814.08, srid: Some(3857) }, Point { x: 520253.11, y: 5529913.55, srid: Some(3857) }, Point { x: 519419.66, y: 5526261.62, srid: Some(3857) }, Point { x: 518486.57, y: 5508201.01, srid: Some(3857) }, Point { x: 518731.17, y: 5506442.46, srid: Some(3857) }, Point { x: 519419.66, y: 5505481.09, srid: Some(3857) }, Point { x: 520443.35, y: 5504722.18, srid: Some(3857) }, Point { x: 521367.39, y: 5503647.17, srid: Some(3857) }, Point { x: 522354.85, y: 5499101.92, srid: Some(3857) }, Point { x: 523659.37, y: 5497578.91, srid: Some(3857) }, Point { x: 524963.90, y: 5495582.33, srid: Some(3857) }, Point { x: 525579.92, y: 5491590.46, srid: Some(3857) }, Point { x: 525154.14, y: 5484514.22, srid: Some(3857) }, Point { x: 525344.38, y: 5481416.53, srid: Some(3857) }, Point { x: 526404.31, y: 5478691.94, srid: Some(3857) }, Point { x: 527165.28, y: 5478691.94, srid: Some(3857) }, Point { x: 529058.66, y: 5480382.10, srid: Some(3857) }, Point { x: 531069.80, y: 5479108.15, srid: Some(3857) }, Point { x: 532917.88, y: 5476888.61, srid: Some(3857) }, Point { x: 534313.00, y: 5475703.39, srid: Some(3857) }, Point { x: 540011.23, y: 5469376.43, srid: Some(3857) }, Point { x: 539993.12, y: 5467587.52, srid: Some(3857) }, Point { x: 539494.86, y: 5466605.03, srid: Some(3857) }, Point { x: 538607.06, y: 5466189.39, srid: Some(3857) }, Point { x: 537384.07, y: 5466094.93, srid: Some(3857) }, Point { x: 536188.25, y: 5465572.26, srid: Some(3857) }, Point { x: 536405.67, y: 5464319.25, srid: Some(3857) }, Point { x: 536922.05, y: 5462739.06, srid: Some(3857) }, Point { x: 536623.09, y: 5461297.61, srid: Some(3857) }, Point { x: 531069.80, y: 5455678.75, srid: Some(3857) }, Point { x: 530127.64, y: 5454961.69, srid: Some(3857) }, Point { x: 529194.55, y: 5454766.71, srid: Some(3857) }, Point { x: 524057.98, y: 5451226.36, srid: Some(3857) }, Point { x: 518241.97, y: 5448259.31, srid: Some(3857) }, Point { x: 517272.63, y: 5447530.26, srid: Some(3857) }, Point { x: 517272.63, y: 5436369.25, srid: Some(3857) }, Point { x: 516665.67, y: 5432886.16, srid: Some(3857) }, Point { x: 513350.00, y: 5424883.19, srid: Some(3857) }, Point { x: 513893.55, y: 5421222.71, srid: Some(3857) }, Point { x: 513839.20, y: 5420539.66, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 514011.32, y: 5420677.52, srid: Some(3857) }, Point { x: 513839.20, y: 5420539.66, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 513839.20, y: 5420539.66, srid: Some(3857) }, Point { x: 512380.66, y: 5419380.48, srid: Some(3857) }, Point { x: 511103.32, y: 5420069.70, srid: Some(3857) }, Point { x: 509200.88, y: 5420583.52, srid: Some(3857) }, Point { x: 502324.95, y: 5420583.52, srid: Some(3857) }, Point { x: 498674.09, y: 5419092.27, srid: Some(3857) }, Point { x: 496672.01, y: 5415621.96, srid: Some(3857) }, Point { x: 495122.88, y: 5411714.73, srid: Some(3857) }, Point { x: 492812.78, y: 5408935.59, srid: Some(3857) }, Point { x: 492812.78, y: 5407978.11, srid: Some(3857) }, Point { x: 494271.32, y: 5407208.44, srid: Some(3857) }, Point { x: 495883.85, y: 5406983.19, srid: Some(3857) }, Point { x: 496916.60, y: 5406445.09, srid: Some(3857) }, Point { x: 496617.65, y: 5404730.92, srid: Some(3857) }, Point { x: 495249.71, y: 5403129.64, srid: Some(3857) }, Point { x: 491970.28, y: 5402091.46, srid: Some(3857) }, Point { x: 490547.98, y: 5400540.66, srid: Some(3857) }, Point { x: 490176.56, y: 5399377.73, srid: Some(3857) }, Point { x: 488482.49, y: 5397889.89, srid: Some(3857) }, Point { x: 484387.73, y: 5395289.88, srid: Some(3857) }, Point { x: 485302.71, y: 5394133.86, srid: Some(3857) }, Point { x: 489922.90, y: 5381208.74, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 513839.20, y: 5420539.66, srid: Some(3857) }, Point { x: 513214.11, y: 5412779.04, srid: Some(3857) }, Point { x: 514554.87, y: 5411132.55, srid: Some(3857) }, Point { x: 516756.26, y: 5409705.41, srid: Some(3857) }, Point { x: 520642.66, y: 5402873.21, srid: Some(3857) }, Point { x: 522599.45, y: 5400540.66, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                           &*format!("{:.2?}", feat.geometry()));
            }
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
    //layer.geometry_type = Some(String::from("MULTIPOLYGON"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let mut ds = GdalDatasource::new("../data/natural_earth.gpkg");
    ds.prepare_queries("ds", &layer, grid.srid);
    let mut reccnt = 0;
    ds.retrieve_features("ds", &layer, &extent, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!("Ok(MultiPolygon(MultiPolygonT { polygons: [PolygonT { rings: [LineStringT { points: [Point { x: 1068024.3649477786, y: 6028202.019",
                       &format!("{:?}", feat.geometry())[0..130]);
        }
        assert_eq!(2, feat.attributes().len());
        assert_eq!(feat.attributes()[0].key, "name");
        assert_eq!(feat.attributes()[1].key, "iso_a3");
        if reccnt == 0 {
            assert_eq!(
                feat.attributes()[0].value,
                FeatureAttrValType::String("Switzerland".to_string())
            );
            assert_eq!(
                feat.attributes()[1].value,
                FeatureAttrValType::String("CHE".to_string())
            );
            assert_eq!(None, feat.fid());
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 1);
}

#[test]
fn test_no_transform() {
    let mut layer = Layer::new("g1k18");
    layer.table_name = Some(String::from("g1k18"));
    let ds = GdalDatasource::new("../data/g1k18.shp");
    let ext = ds.layer_extent(&layer, 3857);
    if gdal_version() < 2030000 {
        assert_eq!(
            format!("{:.5?}", ext),
            "Some(Extent { minx: 5.96526, miny: 45.82056, maxx: 10.56030, maxy: 47.77352 })"
        );
    } else {
        assert_eq!(
            format!("{:.5?}", ext),
            "Some(Extent { minx: 5.96455, miny: 45.81936, maxx: 10.55885, maxy: 47.77213 })"
        );
    }

    layer.no_transform = true;
    let ext = ds.layer_extent(&layer, 3857);
    let extent_fake =
        "Some(Extent { minx: 22.32694, miny: 9.61387, maxx: 25.45679, maxy: 11.56232 })";
    assert_eq!(format!("{:.5?}", ext), extent_fake);
}
