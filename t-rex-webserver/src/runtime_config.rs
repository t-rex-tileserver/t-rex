//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::{Filecache, Nocache, Tilecache};
use crate::core::config::{ApplicationCfg, DEFAULT_CONFIG};
use crate::core::layer::Layer;
use crate::core::{parse_config, read_config, Config};
use crate::datasource::DatasourceType;
use crate::datasources::Datasources;
use crate::mvt_service::MvtService;
use crate::read_qgs;
use crate::service::tileset::Tileset;
use crate::tile_grid::Grid;
use clap::ArgMatches;
use std::process;
use std::str::FromStr;

fn set_layer_buffer_defaults(layer: &mut Layer, simplify: bool, clip: bool) {
    layer.simplify = simplify;
    if simplify {
        // Limit features by default unless simplify is set to false
        layer.query_limit = Some(1000);
        // Set default tolerance
        layer.tolerance = "!pixel_width!/2".to_string();
    }
    layer.buffer_size = match layer.geometry_type {
        Some(ref geom) => {
            if clip {
                if geom.contains("POLYGON") {
                    Some(1)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        }
        None => None,
    };
}

pub fn config_from_args(args: &ArgMatches) -> ApplicationCfg {
    if let Some(cfgpath) = args.value_of("config") {
        info!("Reading configuration from '{}'", cfgpath);
        for argname in vec!["dbconn", "datasource", "qgs"] {
            if args.value_of(argname).is_some() {
                warn!("Ignoring argument `{}`", argname);
            }
        }
        let config = read_config(cfgpath).unwrap_or_else(|err| {
            println!("Error reading configuration - {} ", err);
            process::exit(1)
        });
        config
    } else {
        let bind = args.value_of("bind").unwrap_or("127.0.0.1");
        let port =
            u16::from_str(args.value_of("port").unwrap_or("6767")).expect("Invalid port number");
        let mut config: ApplicationCfg = parse_config(DEFAULT_CONFIG.to_string(), "").unwrap();
        config.webserver.bind = Some(bind.to_string());
        config.webserver.port = Some(port);
        config
    }
}

pub fn service_from_args(config: &ApplicationCfg, args: &ArgMatches) -> MvtService {
    if args.value_of("config").is_some() {
        let mut svc = MvtService::from_config(&config).unwrap_or_else(|err| {
            println!("Error reading configuration - {} ", err);
            process::exit(1)
        });
        svc.connect();
        svc
    } else {
        let cache = match args.value_of("cache") {
            None => Tilecache::Nocache(Nocache),
            Some(dir) => Tilecache::Filecache(Filecache {
                basepath: dir.to_string(),
                baseurl: None,
            }),
        };
        let simplify = bool::from_str(args.value_of("simplify").unwrap_or("true")).unwrap_or(false);
        let clip = bool::from_str(args.value_of("clip").unwrap_or("true")).unwrap_or(false);
        let no_transform =
            bool::from_str(args.value_of("no-transform").unwrap_or("false")).unwrap_or(false);
        let grid = Grid::web_mercator();
        let mut tilesets = Vec::new();
        let datasources = if let Some(qgs) = args.value_of("qgs") {
            info!("Reading configuration from '{}'", qgs);
            let (datasources, mut tileset) = read_qgs(qgs);
            for layer in tileset.layers.iter_mut() {
                set_layer_buffer_defaults(layer, simplify, clip);
            }
            tilesets.push(tileset);
            datasources
        } else {
            let datasources = Datasources::from_args(args);
            if datasources.datasources.is_empty() {
                println!("Either 'config', 'dbconn' or 'datasource' is required");
                process::exit(1)
            }
            let detect_geometry_types =
                bool::from_str(args.value_of("detect-geometry-types").unwrap_or("true"))
                    .unwrap_or(false);
            for (_name, ds) in &datasources.datasources {
                let dsconn = ds.connected();
                let mut layers = dsconn.detect_layers(detect_geometry_types);
                while let Some(mut l) = layers.pop() {
                    l.no_transform = no_transform;
                    let extent = dsconn.layer_extent(&l, 3857);
                    set_layer_buffer_defaults(&mut l, simplify, clip);
                    let tileset = Tileset {
                        name: l.name.clone(),
                        minzoom: None,
                        maxzoom: None,
                        attribution: None,
                        extent: extent,
                        center: None,
                        start_zoom: None,
                        layers: vec![l],
                        cache_limits: None,
                    };
                    tilesets.push(tileset);
                }
            }
            datasources
        };
        let mut svc = MvtService {
            datasources: datasources,
            grid: grid,
            tilesets: tilesets,
            cache: cache,
        };
        svc.connect(); //TODO: ugly - we connect twice
        svc
    }
}

pub fn gen_config(args: &ArgMatches) -> String {
    let toml = r#"
[webserver]
# Bind address. Use 0.0.0.0 to listen on all adresses.
bind = "127.0.0.1"
port = 6767

#[[webserver.static]]
#path = "/static"
#dir = "./public/"
"#;
    let mut config;
    if args.value_of("dbconn").is_some()
        || args.value_of("datasource").is_some()
        || args.value_of("qgs").is_some()
    {
        let service = service_from_args(&config_from_args(args), args);
        config = service.gen_runtime_config();
    } else {
        config = MvtService::gen_config();
    }
    config.push_str(toml);
    config
}

#[test]
fn test_gen_config() {
    use crate::core::parse_config;

    let args = ArgMatches::new();
    let toml = gen_config(&args);
    println!("{}", toml);
    assert_eq!(Some("# t-rex configuration"), toml.lines().next());

    let config = parse_config(toml, "").unwrap();
    let _service = MvtService::from_config(&config).unwrap();
    //assert_eq!(service.input.connection_url,
    //           "postgresql://user:pass@host/database");
}

#[test]
#[ignore]
fn test_runtime_config() {
    use crate::core::parse_config;
    use clap::App;
    use std::env;

    env::var("DBCONN").expect("DBCONN undefined");
    let args = App::new("test")
        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'")
        .get_matches_from(vec!["", "--dbconn", &env::var("DBCONN").unwrap()]);
    let toml = gen_config(&args);
    println!("{}", toml);
    assert_eq!(Some("# t-rex configuration"), toml.lines().next());

    let config = parse_config(toml, "").unwrap();
    let _service = MvtService::from_config(&config).unwrap();
    //assert_eq!(service.input.connection_url, env::var("DBCONN").unwrap());
}
