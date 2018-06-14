//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::feature::FeatureAttrValType;
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use datasource::DatasourceInput;
use gdal;
use gdal::vector::Dataset;
use gdal_ds::GdalDatasource;
use std::path::Path;

#[test]
fn test_gdal_api() {
    let mut dataset = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let layer = dataset.layer_by_name("ne_10m_populated_places").unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap();
    let geometry = feature.geometry();
    assert_eq!(
        name_field.into_string(),
        Some("Colonia del Sacramento".to_string())
    );
    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        geometry.wkt().unwrap(),
        "POINT (-6438719.62282072 -4093437.71441017)".to_string()
    );
    #[cfg(target_os = "macos")]
    assert_eq!(
        geometry.wkt().unwrap(),
        "POINT (-6438719.622820721007884 -4093437.714410172309726)".to_string()
    );
}

#[test]
fn test_detect_layers() {
    let ds = GdalDatasource::new("natural_earth.gpkg");
    let layers = ds.detect_layers(true);
    println!("{:?}", layers);
    assert_eq!(layers.len(), 3);
    assert_eq!(format!("{:?}", layers[0]), r#"Layer { name: "ne_10m_populated_places", datasource: None, geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_10m_populated_places"), query_limit: None, query: [], tile_size: 4096, simplify: false, buffer_size: None, style: None }"#);
    assert_eq!(format!("{:?}", layers[1]), r#"Layer { name: "ne_10m_rivers_lake_centerlines", datasource: None, geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_10m_rivers_lake_centerlines"), query_limit: None, query: [], tile_size: 4096, simplify: false, buffer_size: None, style: None }"#);
    assert_eq!(format!("{:?}", layers[2]), r#"Layer { name: "ne_110m_admin_0_countries", datasource: None, geometry_field: Some("geom"), geometry_type: None, srid: Some(3857), fid_field: None, table_name: Some("ne_110m_admin_0_countries"), query_limit: None, query: [], tile_size: 4096, simplify: false, buffer_size: None, style: None }"#);
}

#[test]
fn test_gdal_retrieve_points() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    //layer.geometry_field = Some(String::from("geom"));
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

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!(
                "Ok(Point(Point { x: 831219.9062494118, y: 5928485.165733484, srid: Some(3857) }))",
                &*format!("{:?}", feat.geometry())
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
    assert_eq!(reccnt, 2);
}

#[test]
fn test_coord_transformation() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    let grid = Grid::wgs84();
    let ds = GdalDatasource::new("natural_earth.gpkg");

    let extent_wgs84 = Extent {
        minx: 7.3828,
        miny: 46.8000,
        maxx: 7.7343,
        maxy: 47.0401,
    };
    #[cfg(not(target_os = "macos"))]
    let extent_3857 = Extent {
        minx: 821849.5366285803,
        miny: 5909489.863677091,
        maxx: 860978.3376424159,
        maxy: 5948621.871058013,
    };
    #[cfg(target_os = "macos")]
    let extent_3857 = Extent {
        minx: 821849.5366285803,
        miny: 5909489.863677091,
        maxx: 860978.3376424166,
        maxy: 5948621.871058013,
    };
    assert_eq!(
        ds.extent_from_wgs84(&extent_wgs84, 3857),
        Some(extent_3857.clone())
    );

    // Invalid input extent doesn't panic
    let result = ds.extent_from_wgs84(&extent_3857, 3857);
    assert!(result.is_none());

    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent_wgs84, 10, &grid, |feat| {
        if reccnt == 0 {
            assert_eq!("Ok(Point(Point { x: 7.466975462482421, y: 46.916682758667704, srid: Some(4326) }))",
                       &*format!("{:?}", feat.geometry()));
        }
        if reccnt == 0 {
            assert_eq!(
                feat.attributes()[1].value,
                FeatureAttrValType::String("Bern".to_string())
            );
        }
        reccnt += 1;
    });
    assert_eq!(reccnt, 2);
}

#[test]
fn test_gdal_retrieve_multilines() {
    let gdal_version = gdal::version::version_info("VERSION_NUM")
        .parse::<i32>()
        .unwrap();
    let mut layer = Layer::new("multilines");
    layer.table_name = Some(String::from("ne_10m_rivers_lake_centerlines"));
    //layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    //layer.geometry_type = Some(String::from("MULTILINE"));
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
    ds.retrieve_features(&layer, &extent, 10, &grid, |_| {
        reccnt += 1;
    });
    assert_eq!(reccnt, 0);

    // with buffer
    layer.buffer_size = Some(600);

    ds.retrieve_features(&layer, &extent, 22, &grid, |_| {
        reccnt += 1;
    });
    assert_eq!(reccnt, 0);

    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
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
            if gdal_version < 2020000 {
                // or maybe < 2000000 ?
                assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 682757.1012729447, y: 5813200.024936108, srid: Some(3857) }, Point { x: 683572.4295746532, y: 5814895.307100639, srid: Some(3857) }, Point { x: 684405.8762830653, y: 5815700.51643066, srid: Some(3857) }, Point { x: 686063.7104965394, y: 5817684.394041292, srid: Some(3857) }, Point { x: 687404.4725926834, y: 5820284.406662052, srid: Some(3857) }, Point { x: 688545.9322150754, y: 5823494.54626182, srid: Some(3857) }, Point { x: 691689.4757783283, y: 5831092.159555616, srid: Some(3857) }, Point { x: 692287.3831995813, y: 5833289.410580246, srid: Some(3857) }, Point { x: 694633.7168678325, y: 5836484.600127116, srid: Some(3857) }, Point { x: 697804.4380411424, y: 5840698.509721698, srid: Some(3857) }, Point { x: 701428.1193820703, y: 5843830.710570353, srid: Some(3857) }, Point { x: 704852.4982492463, y: 5844947.27781533, srid: Some(3857) }, Point { x: 708512.4164035813, y: 5845118.059404142, srid: Some(3857) }, Point { x: 712136.0977445092, y: 5846050.848060609, srid: Some(3857) }, Point { x: 721612.0244510319, y: 5852583.137171743, srid: Some(3857) }, Point { x: 728741.6174893066, y: 5853983.544356565, srid: Some(3857) }, Point { x: 736161.1050348545, y: 5853746.840167029, srid: Some(3857) }, Point { x: 752766.6247796519, y: 5849447.822084563, srid: Some(3857) }, Point { x: 752676.0327461276, y: 5849717.2707096655, srid: Some(3857) }, Point { x: 761236.9799140673, y: 5847509.349466373, srid: Some(3857) }, Point { x: 763248.1230582818, y: 5846142.818490322, srid: Some(3857) }, Point { x: 764335.2274605598, y: 5845131.196586606, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                           &*format!("{:?}", feat.geometry()));
            } else {
                assert_eq!("Ok(MultiLineString(MultiLineStringT { lines: [LineStringT { points: [Point { x: 932700.5217633747, y: 5873592.928376212, srid: Some(3857) }, Point { x: 931649.6541745069, y: 5871280.181806267, srid: Some(3857) }, Point { x: 930263.5960616024, y: 5869969.236968239, srid: Some(3857) }, Point { x: 929312.3797096091, y: 5867874.766288578, srid: Some(3857) }, Point { x: 926784.8619743126, y: 5865240.920736487, srid: Some(3857) }, Point { x: 923134.0030233278, y: 5862239.29927331, srid: Some(3857) }, Point { x: 919628.0913259811, y: 5859034.751478483, srid: Some(3857) }, Point { x: 918441.3356868285, y: 5857969.021057518, srid: Some(3857) }, Point { x: 902406.545753227, y: 5843633.683984192, srid: Some(3857) }, Point { x: 894814.9333439842, y: 5838630.685486072, srid: Some(3857) }, Point { x: 893791.2433651735, y: 5836464.913689302, srid: Some(3857) }, Point { x: 891390.5544768083, y: 5834693.314290535, srid: Some(3857) }, Point { x: 886453.2886497965, y: 5832528.509454539, srid: Some(3857) }, Point { x: 882132.0486507412, y: 5831603.709410817, srid: Some(3857) }, Point { x: 868135.5794714113, y: 5832528.509454539, srid: Some(3857) }, Point { x: 849002.5419913173, y: 5833617.405454226, srid: Some(3857) }, Point { x: 846502.2018660777, y: 5832941.749333624, srid: Some(3857) }, Point { x: 842452.737967592, y: 5829997.017951732, srid: Some(3857) }, Point { x: 832279.2526029388, y: 5827020.493667485, srid: Some(3857) }, Point { x: 827061.151472004, y: 5822328.278913623, srid: Some(3857) }, Point { x: 822794.2666930626, y: 5821135.9625994805, srid: Some(3857) }, Point { x: 819487.657469468, y: 5818450.565059974, srid: Some(3857) }, Point { x: 800979.7050206838, y: 5809444.071788309, srid: Some(3857) }, Point { x: 790597.8579789284, y: 5801544.765992771, srid: Some(3857) }, Point { x: 787227.8343318665, y: 5800923.8448735615, srid: Some(3857) }, Point { x: 786249.4403698161, y: 5801845.438194744, srid: Some(3857) }, Point { x: 784990.2111038431, y: 5803675.837272042, srid: Some(3857) }, Point { x: 783903.106701565, y: 5805663.556672839, srid: Some(3857) }, Point { x: 783441.0873305968, y: 5807010.752106182, srid: Some(3857) }, Point { x: 783341.4360937224, y: 5810346.925245712, srid: Some(3857) }, Point { x: 782951.8903495717, y: 5812467.030672076, srid: Some(3857) }, Point { x: 782118.4436411596, y: 5814050.898104682, srid: Some(3857) }, Point { x: 779618.10351592, y: 5817409.374502269, srid: Some(3857) }, Point { x: 772551.9249011126, y: 5829760.957335796, srid: Some(3857) }, Point { x: 772035.5503100306, y: 5831229.881597894, srid: Some(3857) }, Point { x: 771546.3533290053, y: 5834076.61877609, srid: Some(3857) }, Point { x: 770350.5384864995, y: 5835027.922461492, srid: Some(3857) }, Point { x: 768855.7699333671, y: 5835342.859400253, srid: Some(3857) }, Point { x: 767469.7118204626, y: 5836287.737727394, srid: Some(3857) }, Point { x: 767052.988466255, y: 5837488.6667607175, srid: Some(3857) }, Point { x: 767188.8765165397, y: 5838689.759445391, srid: Some(3857) }, Point { x: 767460.6526171094, y: 5839772.852217527, srid: Some(3857) }, Point { x: 767469.7118204626, y: 5840600.0308968015, srid: Some(3857) }, Point { x: 766763.0939589819, y: 5841978.834620021, srid: Some(3857) }, Point { x: 765105.2597455079, y: 5843935.793218367, srid: Some(3857) }, Point { x: 764362.4050706167, y: 5845104.9222412715, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 682757.1012729447, y: 5813200.024936108, srid: Some(3857) }, Point { x: 681534.1088203818, y: 5813304.743366074, srid: Some(3857) }, Point { x: 680003.1034538391, y: 5813684.358078509, srid: Some(3857) }, Point { x: 678888.821441504, y: 5814194.900132486, srid: Some(3857) }, Point { x: 678318.0916303082, y: 5814378.178835105, srid: Some(3857) }, Point { x: 678182.2035800235, y: 5814541.8237467, srid: Some(3857) }, Point { x: 677049.8031609848, y: 5814234.173820129, srid: Some(3857) }, Point { x: 675935.5211486497, y: 5813854.535485366, srid: Some(3857) }, Point { x: 674794.0615262578, y: 5813671.26764451, srid: Some(3857) }, Point { x: 673571.069073695, y: 5813200.024936108, srid: Some(3857) }, Point { x: 670146.6902065191, y: 5809797.351289222, srid: Some(3857) }, Point { x: 668108.3694522477, y: 5808534.769123822, srid: Some(3857) }, Point { x: 664321.6224509781, y: 5802682.145063239, srid: Some(3857) }, Point { x: 662863.090711256, y: 5800923.8448735615, srid: Some(3857) }, Point { x: 661775.986308978, y: 5800747.380507665, srid: Some(3857) }, Point { x: 659384.3566239662, y: 5801139.528322787, srid: Some(3857) }, Point { x: 658315.3706283919, y: 5800923.8448735615, srid: Some(3857) }, Point { x: 657400.3910898089, y: 5800021.952865967, srid: Some(3857) }, Point { x: 656005.273773551, y: 5797336.427620337, srid: Some(3857) }, Point { x: 655253.3598953097, y: 5796278.123280324, srid: Some(3857) }, Point { x: 653622.7032918925, y: 5795317.919568167, srid: Some(3857) }, Point { x: 652743.9605667167, y: 5795944.979580933, srid: Some(3857) }, Point { x: 651992.0466884755, y: 5797199.232882183, srid: Some(3857) }, Point { x: 650769.0542359126, y: 5798159.640712233, srid: Some(3857) }, Point { x: 648458.9573810718, y: 5795631.444022353, srid: Some(3857) }, Point { x: 648350.246940844, y: 5788403.608999123, srid: Some(3857) }, Point { x: 650008.081154318, y: 5775633.835183917, srid: Some(3857) }, Point { x: 650171.1468146597, y: 5769607.516746944, srid: Some(3857) }, Point { x: 649210.8712593131, y: 5764053.897571404, srid: Some(3857) }, Point { x: 645605.3083250918, y: 5753444.617301839, srid: Some(3857) }, Point { x: 644762.8024133263, y: 5750129.781392407, srid: Some(3857) }, Point { x: 644409.493482586, y: 5747868.602222382, srid: Some(3857) }, Point { x: 642914.7249294536, y: 5747634.719934073, srid: Some(3857) }, Point { x: 641012.292225467, y: 5747634.719934073, srid: Some(3857) }, Point { x: 638511.9521002275, y: 5747166.973800436, srid: Some(3857) }, Point { x: 636799.7626666396, y: 5744750.677051108, srid: Some(3857) }, Point { x: 635413.704553735, y: 5741510.491582338, srid: Some(3857) }, Point { x: 635486.1781805524, y: 5737836.677909708, srid: Some(3857) }, Point { x: 636011.6119749879, y: 5734508.187267991, srid: Some(3857) }, Point { x: 635957.256754874, y: 5732426.079818112, srid: Some(3857) }, Point { x: 633864.5807804888, y: 5730344.458706714, srid: Some(3857) }, Point { x: 630132.1889993331, y: 5725560.518151296, srid: Some(3857) }, Point { x: 627939.8617880733, y: 5723765.58208322, srid: Some(3857) }, Point { x: 625602.5873231755, y: 5723441.624883558, srid: Some(3857) }, Point { x: 623446.4969253229, y: 5725327.220837952, srid: Some(3857) }, Point { x: 619596.3355005892, y: 5732620.648692069, srid: Some(3857) }, Point { x: 617494.6003228506, y: 5735520.228278014, srid: Some(3857) }, Point { x: 609776.1590666763, y: 5742627.215902862, srid: Some(3857) }, Point { x: 606714.1483335942, y: 5747238.433423944, srid: Some(3857) }, Point { x: 604802.6564262542, y: 5753321.101253894, srid: Some(3857) }, Point { x: 604947.6036798923, y: 5753464.119992565, srid: Some(3857) }, Point { x: 605300.9126106327, y: 5753600.64002539, srid: Some(3857) }, Point { x: 605463.9782709744, y: 5753737.162154317, srid: Some(3857) }, Point { x: 605020.0773067098, y: 5754621.356215678, srid: Some(3857) }, Point { x: 604485.5843089242, y: 5755427.609798645, srid: Some(3857) }, Point { x: 599448.6672450347, y: 5760162.5741662355, srid: Some(3857) }, Point { x: 597682.1225913329, y: 5761131.991979023, srid: Some(3857) }, Point { x: 596893.9718996813, y: 5761014.875485132, srid: Some(3857) }, Point { x: 596350.4196985423, y: 5760487.870366003, srid: Some(3857) }, Point { x: 595236.1376862072, y: 5760162.5741662355, srid: Some(3857) }, Point { x: 593723.2507263713, y: 5759219.282524907, srid: Some(3857) }, Point { x: 592083.5349196008, y: 5756988.308361211, srid: Some(3857) }, Point { x: 586974.1442288939, y: 5748329.888089515, srid: Some(3857) }, Point { x: 585243.8363886024, y: 5746718.740161177, srid: Some(3857) }, Point { x: 583223.6340410346, y: 5746069.166262354, srid: Some(3857) }, Point { x: 581076.6028465355, y: 5746575.829834943, srid: Some(3857) }, Point { x: 577869.6448598151, y: 5748823.685381301, srid: Some(3857) }, Point { x: 575179.061464177, y: 5749798.365740731, srid: Some(3857) }, Point { x: 571491.9656997849, y: 5751982.0373877315, srid: Some(3857) }, Point { x: 569779.7762661971, y: 5752606.042062425, srid: Some(3857) }, Point { x: 568004.1724091418, y: 5752541.039532486, srid: Some(3857) }, Point { x: 564099.6557642941, y: 5751657.052292971, srid: Some(3857) }, Point { x: 562197.2230603076, y: 5751514.062613355, srid: Some(3857) }, Point { x: 555728.951866753, y: 5752606.042062425, srid: Some(3857) }, Point { x: 553880.8743828803, y: 5752281.034164034, srid: Some(3857) }, Point { x: 550692.0348028637, y: 5750851.140498335, srid: Some(3857) }, Point { x: 547349.1887658585, y: 5750168.772280888, srid: Some(3857) }, Point { x: 542946.4159366325, y: 5748225.934257235, srid: Some(3857) }, Point { x: 541596.5946371383, y: 5747147.48491181, srid: Some(3857) }, Point { x: 541179.8712829306, y: 5744887.063428765, srid: Some(3857) }, Point { x: 541134.5752661701, y: 5742003.910637028, srid: Some(3857) }, Point { x: 540563.8454549741, y: 5740367.941904743, srid: Some(3857) }, Point { x: 538570.8207174633, y: 5741796.151912501, srid: Some(3857) }, Point { x: 537737.3740090511, y: 5740757.431005507, srid: Some(3857) }, Point { x: 537048.8745542739, y: 5739524.107293887, srid: Some(3857) }, Point { x: 539195.9057487731, y: 5735169.893380078, srid: Some(3857) }, Point { x: 538842.5968180328, y: 5729722.012236262, srid: Some(3857) }, Point { x: 536912.9865039892, y: 5724141.387175135, srid: Some(3857) }, Point { x: 534312.9951418752, y: 5719347.819619257, srid: Some(3857) }, Point { x: 534602.8896491483, y: 5715359.461406825, srid: Some(3857) }, Point { x: 538706.708767748, y: 5712175.233862496, srid: Some(3857) }, Point { x: 542131.087634924, y: 5708474.672324258, srid: Some(3857) }, Point { x: 540427.9574046893, y: 5702965.487067183, srid: Some(3857) }, Point { x: 532818.2265887429, y: 5696309.851389048, srid: Some(3857) }, Point { x: 529883.0447025921, y: 5691763.603939567, srid: Some(3857) }, Point { x: 529366.6701115101, y: 5685038.876076876, srid: Some(3857) }, Point { x: 531377.8132557244, y: 5677842.170958379, srid: Some(3857) }, Point { x: 531649.589356294, y: 5674761.477894884, srid: Some(3857) }, Point { x: 532492.0952680595, y: 5672815.64403906, srid: Some(3857) }, Point { x: 534222.403108351, y: 5670999.054402079, srid: Some(3857) }, Point { x: 535617.520424609, y: 5668770.691523319, srid: Some(3857) }, Point { x: 535526.9283910847, y: 5665570.803000479, srid: Some(3857) }, Point { x: 536215.4278458619, y: 5665570.803000479, srid: Some(3857) }, Point { x: 535726.2308648367, y: 5662577.976056778, srid: Some(3857) }, Point { x: 535617.520424609, y: 5657598.578159716, srid: Some(3857) }, Point { x: 535907.4149318819, y: 5652647.650480549, srid: Some(3857) }, Point { x: 537375.0058749574, y: 5646414.646060817, srid: Some(3857) }, Point { x: 536831.4536738184, y: 5642445.766811948, srid: Some(3857) }, Point { x: 536867.6904872287, y: 5639043.429143266, srid: Some(3857) }, Point { x: 539313.6753923544, y: 5637438.999118198, srid: Some(3857) }, Point { x: 538906.0112415, y: 5635437.071388837, srid: Some(3857) }, Point { x: 539440.5042392857, y: 5634243.826164, srid: Some(3857) }, Point { x: 540283.0101510512, y: 5633255.9900475815, srid: Some(3857) }, Point { x: 540835.6215555436, y: 5631857.809812354, srid: Some(3857) }, Point { x: 540853.7399622472, y: 5629998.181514638, srid: Some(3857) }, Point { x: 540183.3589141767, y: 5626760.778918007, srid: Some(3857) }, Point { x: 540011.2340504817, y: 5624915.018309012, srid: Some(3857) }, Point { x: 540726.9111153157, y: 5621884.432441977, srid: Some(3857) }, Point { x: 543897.6322886257, y: 5616287.253539882, srid: Some(3857) }, Point { x: 544640.4869635168, y: 5613579.567578901, srid: Some(3857) }, Point { x: 544196.5859992523, y: 5610859.896933747, srid: Some(3857) }, Point { x: 541968.0219745822, y: 5603408.996048684, srid: Some(3857) }, Point { x: 541216.1080963409, y: 5601874.809408514, srid: Some(3857) }, Point { x: 540799.3847421332, y: 5601254.816186602, srid: Some(3857) }, Point { x: 538326.2222269507, y: 5598577.189503336, srid: Some(3857) }, Point { x: 537737.3740090511, y: 5598078.816695047, srid: Some(3857) }, Point { x: 537483.7163151852, y: 5595900.355419405, srid: Some(3857) }, Point { x: 536813.3352671148, y: 5594916.706144168, srid: Some(3857) }, Point { x: 535853.0597117681, y: 5594316.349407393, srid: Some(3857) }, Point { x: 534693.4816826725, y: 5593307.328894206, srid: Some(3857) }, Point { x: 533026.5882658452, y: 5591289.625349757, srid: Some(3857) }, Point { x: 531839.8326266926, y: 5589374.50015909, srid: Some(3857) }, Point { x: 531133.2147652119, y: 5586853.536415065, srid: Some(3857) }, Point { x: 530888.6162746993, y: 5582987.295052161, srid: Some(3857) }, Point { x: 531323.4580356105, y: 5580914.495384848, srid: Some(3857) }, Point { x: 533352.7195865285, y: 5576413.369728287, srid: Some(3857) }, Point { x: 533932.5086010778, y: 5573845.0319545455, srid: Some(3857) }, Point { x: 533162.4763161299, y: 5567914.505053018, srid: Some(3857) }, Point { x: 530616.8401741298, y: 5563687.151322663, srid: Some(3857) }, Point { x: 527654.4806779221, y: 5559748.062003214, srid: Some(3857) }, Point { x: 525579.9231102405, y: 5554615.173503827, srid: Some(3857) }, Point { x: 523577.8391693793, y: 5539202.133223146, srid: Some(3857) }, Point { x: 523161.11581517174, y: 5537767.622019857, srid: Some(3857) }, Point { x: 520642.6572832286, y: 5533351.254332017, srid: Some(3857) }, Point { x: 520099.1050820896, y: 5532000.12133614, srid: Some(3857) }, Point { x: 519899.8026083375, y: 5530814.080518678, srid: Some(3857) }, Point { x: 520253.1115390779, y: 5529913.553699311, srid: Some(3857) }, Point { x: 519419.66483066574, y: 5526261.624674759, srid: Some(3857) }, Point { x: 518486.5668853761, y: 5508201.0129065495, srid: Some(3857) }, Point { x: 518731.16537588864, y: 5506442.455889619, srid: Some(3857) }, Point { x: 519419.66483066574, y: 5505481.0857884865, srid: Some(3857) }, Point { x: 520443.3548094765, y: 5504722.1808216, srid: Some(3857) }, Point { x: 521367.3935514129, y: 5503647.173305425, srid: Some(3857) }, Point { x: 522354.8467168166, y: 5499101.921234139, srid: Some(3857) }, Point { x: 523659.37199955026, y: 5497578.913754652, srid: Some(3857) }, Point { x: 524963.897282284, y: 5495582.32542858, srid: Some(3857) }, Point { x: 525579.9231102405, y: 5491590.455264605, srid: Some(3857) }, Point { x: 525154.1405526826, y: 5484514.217656136, srid: Some(3857) }, Point { x: 525344.3838230813, y: 5481416.533673438, srid: Some(3857) }, Point { x: 526404.3106153023, y: 5478691.942095304, srid: Some(3857) }, Point { x: 527165.283696897, y: 5478691.942095304, srid: Some(3857) }, Point { x: 529058.6571975301, y: 5480382.10261068, srid: Some(3857) }, Point { x: 531069.8003417447, y: 5479108.146733513, srid: Some(3857) }, Point { x: 532917.8778256173, y: 5476888.606969105, srid: Some(3857) }, Point { x: 534312.9951418752, y: 5475703.391030162, srid: Some(3857) }, Point { x: 540011.2340504817, y: 5469376.425068476, srid: Some(3857) }, Point { x: 539993.1156437781, y: 5467587.516803934, srid: Some(3857) }, Point { x: 539494.8594593997, y: 5466605.025382432, srid: Some(3857) }, Point { x: 538607.0575308736, y: 5466189.38758625, srid: Some(3857) }, Point { x: 537384.0650783108, y: 5466094.927074974, srid: Some(3857) }, Point { x: 536188.2502358048, y: 5465572.26314542, srid: Some(3857) }, Point { x: 536405.6711162605, y: 5464319.250312821, srid: Some(3857) }, Point { x: 536922.0457073427, y: 5462739.060826276, srid: Some(3857) }, Point { x: 536623.0919967161, y: 5461297.611135728, srid: Some(3857) }, Point { x: 531069.8003417447, y: 5455678.7462252155, srid: Some(3857) }, Point { x: 530127.6431931047, y: 5454961.691523765, srid: Some(3857) }, Point { x: 529194.545247815, y: 5454766.712636923, srid: Some(3857) }, Point { x: 524057.9769470511, y: 5451226.36371692, srid: Some(3857) }, Point { x: 518241.9683948635, y: 5448259.305806925, srid: Some(3857) }, Point { x: 517272.6336361666, y: 5447530.260077521, srid: Some(3857) }, Point { x: 517272.6336361666, y: 5436369.245221924, srid: Some(3857) }, Point { x: 516665.6670115603, y: 5432886.16074568, srid: Some(3857) }, Point { x: 513349.9985846122, y: 5424883.191885986, srid: Some(3857) }, Point { x: 513893.5507857512, y: 5421222.706015395, srid: Some(3857) }, Point { x: 513839.1955656373, y: 5420539.659649931, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 514011.32042933244, y: 5420677.518147655, srid: Some(3857) }, Point { x: 513839.1955656373, y: 5420539.659649931, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 513839.1955656373, y: 5420539.659649931, srid: Some(3857) }, Point { x: 512380.6638259153, y: 5419380.476454268, srid: Some(3857) }, Point { x: 511103.31615323864, y: 5420069.702971984, srid: Some(3857) }, Point { x: 509200.883449252, y: 5420583.52349405, srid: Some(3857) }, Point { x: 502324.9481048432, y: 5420583.52349405, srid: Some(3857) }, Point { x: 498674.0891538584, y: 5419092.269714077, srid: Some(3857) }, Point { x: 496672.0052129973, y: 5415621.964525247, srid: Some(3857) }, Point { x: 495122.8814397511, y: 5411714.73396912, srid: Some(3857) }, Point { x: 492812.78458491014, y: 5408935.594904593, srid: Some(3857) }, Point { x: 492812.78458491014, y: 5407978.112172775, srid: Some(3857) }, Point { x: 494271.31632463227, y: 5407208.443058552, srid: Some(3857) }, Point { x: 495883.85452134575, y: 5406983.186170905, srid: Some(3857) }, Point { x: 496916.60370350984, y: 5406445.094709369, srid: Some(3857) }, Point { x: 496617.6499928834, y: 5404730.919076169, srid: Some(3857) }, Point { x: 495249.7102866825, y: 5403129.640590461, srid: Some(3857) }, Point { x: 491970.2786731447, y: 5402091.459715059, srid: Some(3857) }, Point { x: 490547.98374682985, y: 5400540.659527809, srid: Some(3857) }, Point { x: 490176.55640938587, y: 5399377.729947131, srid: Some(3857) }, Point { x: 488482.4853825015, y: 5397889.893382944, srid: Some(3857) }, Point { x: 484387.7254672552, y: 5395289.87933264, srid: Some(3857) }, Point { x: 485302.7050058382, y: 5394133.857618519, srid: Some(3857) }, Point { x: 489922.89871552, y: 5381208.739980754, srid: Some(3857) }], srid: Some(3857) }, LineStringT { points: [Point { x: 513839.1955656373, y: 5420539.659649931, srid: Some(3857) }, Point { x: 513214.11053432745, y: 5412779.040064176, srid: Some(3857) }, Point { x: 514554.8726304715, y: 5411132.547802317, srid: Some(3857) }, Point { x: 516756.25904508453, y: 5409705.40784984, srid: Some(3857) }, Point { x: 520642.6572832286, y: 5402873.211586583, srid: Some(3857) }, Point { x: 522599.4452073291, y: 5400540.659527809, srid: Some(3857) }], srid: Some(3857) }], srid: Some(3857) }))",
                           &*format!("{:?}", feat.geometry()));
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
    //layer.geometry_field = Some(String::from("geom"));
    layer.srid = Some(3857);
    //layer.geometry_type = Some(String::from("MULTIPOLYGON"));
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
