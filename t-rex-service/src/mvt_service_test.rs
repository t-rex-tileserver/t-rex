//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::{DatasourceInput, PostgisInput};
use datasource_type::{Datasource, Datasources};
use core::grid::Grid;
use core::grid::Extent;
use core::layer::Layer;
use core::Config;
use cache::{Nocache, Tilecache};
use service::tileset::Tileset;
use mvt_service::MvtService;

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
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    layer.query_limit = Some(1);
    let tileset = Tileset {
        name: "points".to_string(),
        attribution: Some("Attribution".to_string()),
        extent: Some(Extent {
            minx: -179.58998,
            miny: -90.00000,
            maxx: 179.38330,
            maxy: 82.48332,
        }),
        layers: vec![layer],
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

    let mvt_tile = service.tile("points", 33, 41, 6);
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
                    double_value: Some(
                        10
                    ),
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
                    double_value: Some(
                        11315
                    ),
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
        minx: 445277.9631730949,
        miny: 6800125.454397307,
        maxx: 556597.4539663672,
        maxy: 6982997.920389788,
    };
    #[cfg(target_os = "macos")]
    let extent_3857 = Extent {
        minx: 445277.9631730949,
        miny: 6800125.454397305,
        maxx: 556597.4539663672,
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
fn test_mvt_metadata() {
    use core::read_config;

    let config = read_config("src/test/example.toml").unwrap();
    let service = MvtService::from_config(&config).unwrap();

    let metadata = format!("{:#}", service.get_mvt_metadata().unwrap());
    let expected = r#"{
  "tilesets": [
    {
      "bounds": [
        -180.0,
        -90.0,
        180.0,
        90.0
      ],
      "layers": [
        {
          "geometry_type": "POINT",
          "name": "points"
        },
        {
          "geometry_type": "POLYGON",
          "name": "buildings"
        },
        {
          "geometry_type": "POLYGON",
          "name": "admin_0_countries"
        }
      ],
      "name": "osm",
      "supported": true,
      "tilejson": "osm.json",
      "tileurl": "/osm/{z}/{x}/{y}.pbf"
    }
  ]
}"#;
    println!("{}", metadata);
    assert_eq!(metadata, expected);
}

#[test]
#[ignore]
fn test_tilejson() {
    use core::read_config;
    use std::env;

    match env::var("DBCONN") {
        Err(_) => panic!("DBCONN undefined"),
        // Overwrite PG connection in example.toml
        Ok(dbconn) => env::set_var("TREX_DATASOURCE_URL", dbconn),
    }

    let config = read_config("src/test/example.toml").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    service.prepare_feature_queries();
    let metadata = format!(
        "{:#}",
        service.get_tilejson("http://127.0.0.1", "osm").unwrap()
    );
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": [
    -180.0,
    -90.0,
    180.0,
    90.0
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "maxzoom": 22,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "tiles": [
    "http://127.0.0.1/osm/{z}/{x}/{y}.pbf"
  ],
  "vector_layers": [
    {
      "description": "",
      "fields": {},
      "id": "points",
      "maxzoom": 22,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "buildings",
      "maxzoom": 22,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "admin_0_countries",
      "maxzoom": 22,
      "minzoom": 0
    }
  ],
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
fn test_stylejson() {
    use core::read_config;

    let config = read_config("src/test/example.toml").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let json = format!(
        "{:#}",
        service.get_stylejson("http://127.0.0.1", "osm").unwrap()
    );
    println!("{}", json);
    let expected = r#"
  "name": "t-rex",
  "sources": {
    "osm": {
      "type": "vector",
      "url": "http://127.0.0.1/osm.json"
    }
  },
  "version": 8
"#;
    assert!(json.contains(expected));
    let expected = r#"
  "layers": [
    {
      "id": "background_",
      "paint": {
        "background-color": "rgba(255, 255, 255, 1)"
      },
      "type": "background"
    },
    {
      "id": "points","#;
    assert!(json.contains(expected));

    let expected = r##"
      "paint": {
        "fill-color": "#d8e8c8",
        "fill-opacity": 0.5
      },"##;
    assert!(json.contains(expected));

    let expected = r#"
      "id": "buildings","#;
    assert!(json.contains(expected));
}

#[test]
#[ignore]
fn test_mbtiles_metadata() {
    use core::read_config;
    use std::env;

    match env::var("DBCONN") {
        Err(_) => panic!("DBCONN undefined"),
        // Overwrite PG connection in example.toml
        Ok(dbconn) => env::set_var("TREX_DATASOURCE_URL", dbconn),
    }

    let config = read_config("src/test/example.toml").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    let metadata = format!("{:#}", service.get_mbtiles_metadata("osm").unwrap());
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": "[-180.0,-90.0,180.0,90.0]",
  "center": "[0.0,0.0,2]",
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "json": "{\"Layer\":[{\"description\":\"\",\"fields\":{},\"id\":\"points\",\"name\":\"points\",\"properties\":{\"buffer-size\":0,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"name\":\"buildings\",\"properties\":{\"buffer-size\":10,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"admin_0_countries\",\"name\":\"admin_0_countries\",\"properties\":{\"buffer-size\":1,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"}],\"vector_layers\":[{\"description\":\"\",\"fields\":{},\"id\":\"points\",\"maxzoom\":22,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"maxzoom\":22,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"admin_0_countries\",\"maxzoom\":22,\"minzoom\":0}]}",
  "maxzoom": 22,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
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

[[tileset.layer]]
name = "points"
table_name = "mytable"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
#fid_field = "id"
#tile_size = 4096
#simplify = true
#buffer_size = 10
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
