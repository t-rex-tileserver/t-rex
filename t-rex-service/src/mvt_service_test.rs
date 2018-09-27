//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use cache::{Nocache, Tilecache};
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use core::Config;
use datasource::{DatasourceInput, PostgisInput};
use datasource_type::{Datasource, Datasources};
use mvt_service::MvtService;
use service::tileset::Tileset;

fn mvt_service() -> MvtService {
    use std::env;

    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput::new(&val).connected()),
        Result::Err(_) => panic!("DBCONN undefined"),
    }.unwrap();
    let mut datasources = Datasources::new();
    datasources.add(&"pg".to_string(), Datasource::Postgis(pg));
    datasources.setup();
    let grid = Grid::web_mercator();
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne.ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    layer.query_limit = Some(1);
    let tileset = Tileset {
        name: "points".to_string(),
        minzoom: Some(0),
        maxzoom: Some(22),
        center: None,
        start_zoom: Some(3),
        attribution: Some("Attribution".to_string()),
        extent: Some(Extent {
            minx: -179.58998,
            miny: -90.00000,
            maxx: 179.38330,
            maxy: 82.48332,
        }),
        layers: vec![layer],
        cache_limits: None
    };
    let mut service = MvtService {
        datasources: datasources,
        grid: grid,
        tilesets: vec![tileset],
        cache: Tilecache::Nocache(Nocache),
    };
    service.prepare_feature_queries();
    service
}

#[test]
#[ignore]
fn test_tile_query() {
    let service = mvt_service();

    let mvt_tile = service.tile("points", 33, 41, 6, None);
    println!("{:#?}", mvt_tile);
    let expected = r#"Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some("points"),
            features: [
                Tile_Feature {
                    id: None,
                    tags: [
                        0,
                        0,
                        1,
                        1,
                        2,
                        2,
                        3,
                        3
                    ],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        2504,
                        3390
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            keys: [
                "fid",
                "scalerank",
                "name",
                "pop_max"
            ],
            values: [
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        106
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        10
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: Some("Delemont"),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                },
                Tile_Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        11315
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell {
                        value: 0
                    }
                }
            ],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell {
                value: 0
            }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell {
        value: 0
    }
}"#;
    assert_eq!(expected, &*format!("{:#.0?}", mvt_tile));
}

#[test]
#[ignore]
fn test_projected_extent() {
    let service = mvt_service();

    let extent_wgs84 = Extent {
        minx: 4.0,
        miny: 52.0,
        maxx: 5.0,
        maxy: 53.0,
    };
    #[cfg(not(target_os = "macos"))]
    let extent_3857 = Extent {
        minx: 445277.96317309426,
        miny: 6800125.454397307,
        maxx: 556597.4539663679,
        maxy: 6982997.920389788,
    };
    #[cfg(target_os = "macos")]
    let extent_3857 = Extent {
        minx: 445277.96317309426,
        miny: 6800125.454397305,
        maxx: 556597.4539663679,
        maxy: 6982997.920389788,
    };

    assert_eq!(service.extent_from_wgs84(&extent_wgs84), extent_3857);
}

#[test]
#[ignore]
fn test_generate() {
    let service = mvt_service();

    // Single tile level 23
    let extent = Extent {
        minx: 9.43743,
        miny: 47.05001,
        maxx: 9.43751,
        maxy: 47.05006,
    };

    assert_eq!(service.grid.maxzoom(), 22);
    service.generate(
        Some("points"),
        Some(20),
        Some(23),
        Some(extent),
        None,
        None,
        false,
        false,
    );
}

#[test]
fn test_gen_config() {
    #[cfg(feature = "with-gdal")]
    let gdal_ds_cfg = r#"
[[datasource]]
name = "ds"
# Dataset specification (http://gdal.org/ogr_formats.html)
path = "<filename-or-connection-spec>"
"#;
    #[cfg(not(feature = "with-gdal"))]
    let gdal_ds_cfg = "";

    let expected = format!(
        r#"# t-rex configuration

[service.mvt]
viewer = true

[[datasource]]
name = "database"
# PostgreSQL connection specification (https://github.com/sfackler/rust-postgres#connecting)
dbconn = "postgresql://user:pass@host/database"
{}
[grid]
# Predefined grids: web_mercator, wgs84
predefined = "web_mercator"

[[tileset]]
name = "points"
#minzoom = 0
#maxzoom = 22
#attribution = "© Contributeurs de OpenStreetMap" # Acknowledgment of ownership, authorship or copyright.
#cache_limits = {minzoom = 0, maxzoom = 22, no_cache = false}

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#simplify = true
#tolerance = "!pixel_width!/2"
#buffer_size = 10
#make_valid = true
#[[tileset.layer.query]]
#minzoom = 0
#maxzoom = 22
#sql = "SELECT name,wkb_geometry FROM mytable"

#[cache.file]
#base = "/tmp/mvtcache"
#baseurl = "http://example.com/tiles"
"#,
        gdal_ds_cfg
    );
    println!("{}", &MvtService::gen_config());
    assert_eq!(&expected, &MvtService::gen_config());
}
