//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate clap;

use clap::{App, AppSettings, ArgMatches, SubCommand};
use dotenv::dotenv;
use env_logger::Builder;
use log::Record;
use std::env;
use std::io::Write;
use t_rex_webserver as webserver;
use tile_grid::Extent;
use time;

fn init_logger(args: &ArgMatches<'_>) {
    let mut builder = Builder::new();
    builder.format(|buf, record: &Record<'_>| {
        let t = time::now();
        writeln!(
            buf,
            "{}.{:03} {} {}",
            time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
            t.tm_nsec / 1000_000,
            record.level(),
            record.args()
        )
    });

    let rust_log_env = env::var("RUST_LOG");
    let rust_log = if args.value_of("loglevel").is_none() && rust_log_env.is_ok() {
        rust_log_env.as_ref().unwrap()
    } else {
        match args.value_of("loglevel").unwrap_or("info") {
            "debug" => "debug,tokio=info",
            loglevel => loglevel,
        }
    };
    builder.parse_filters(rust_log);

    builder.init();
}

fn generate(args: &ArgMatches<'_>) {
    let config = webserver::config_from_args(&args);
    let mut service = webserver::service_from_args(&config, &args);
    config
        .cache
        .expect("Missing configuration entry base in [cache.file]");
    let tileset = args.value_of("tileset");
    let minzoom = args.value_of("minzoom").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'minzoom' as integer value")
    });
    let maxzoom = args.value_of("maxzoom").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'maxzoom' as integer value")
    });
    let extent = args.value_of("extent").and_then(|numlist| {
        let arr: Vec<f64> = numlist
            .split(",")
            .map(|v| {
                v.parse()
                    .expect("Error parsing 'extent' as list of float values")
            })
            .collect();
        Some(Extent {
            minx: arr[0],
            miny: arr[1],
            maxx: arr[2],
            maxy: arr[3],
        })
    });

    let extent_srid = args.value_of("extent").and_then(|numlist| {
        let arr: Vec<&str> = numlist.split(",").collect();
        match arr.len() {
            5 => {
                let srid = arr[4];
                let srid_int: i32 = srid
                    .parse()
                    .expect("Error parsing 'srid' in 'extent' as integer");
                Some(srid_int)
            }
            _ => None,
        }
    });
    let nodes = args.value_of("nodes").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'nodes' as integer value")
    });
    let nodeno = args.value_of("nodeno").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'nodeno' as integer value")
    });
    let progress = args.value_of("progress").map_or(true, |s| {
        s.parse::<bool>()
            .expect("Error parsing 'progress' as boolean value")
    });
    let overwrite = args.value_of("overwrite").map_or(false, |s| {
        s.parse::<bool>()
            .expect("Error parsing 'overwrite' as boolean value")
    });
    service.prepare_feature_queries();
    service.generate(
        tileset,
        minzoom,
        maxzoom,
        extent,
        nodes,
        nodeno,
        progress,
        overwrite,
        extent_srid,
    );
}

fn drilldown(args: &ArgMatches<'_>) {
    let config = webserver::config_from_args(&args);
    let mut service = webserver::service_from_args(&config, &args);
    let tileset = args.value_of("tileset");
    let minzoom = args.value_of("minzoom").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'minzoom' as integer value")
    });
    let maxzoom = args.value_of("maxzoom").map(|s| {
        s.parse::<u8>()
            .expect("Error parsing 'maxzoom' as integer value")
    });
    let points: Vec<f64> = args
        .value_of("points")
        .map(|numlist| {
            numlist
                .split(",")
                .map(|v| {
                    v.parse()
                        .expect("Error parsing 'point' as pair of float values")
                })
                .collect()
        })
        .expect("Missing 'points' list");
    let progress = args.value_of("progress").map_or(true, |s| {
        s.parse::<bool>()
            .expect("Error parsing 'progress' as boolean value")
    });
    service.prepare_feature_queries();
    let stats = service.drilldown(tileset, minzoom, maxzoom, points, progress);
    print!("{}", stats.as_csv());
}

#[cfg(feature = "with-gdal")]
extern crate t_rex_gdal;

fn version_info() -> String {
    #[cfg(feature = "with-gdal")]
    let version = format!(
        "{} (GDAL version {})",
        crate_version!(),
        t_rex_gdal::gdal_version()
    );
    #[cfg(not(feature = "with-gdal"))]
    let version = crate_version!().to_string();
    version
}

fn main() {
    dotenv().ok();
    let version_info = version_info();
    // http://kbknapp.github.io/clap-rs/clap/
    let mut app = App::new("t_rex")
        .version(&version_info as &str)
        .author("Pirmin Kalberer <pka@sourcepole.ch>")
        .about("vector tile server specialized on publishing MVT tiles from your own data")
        .subcommand(SubCommand::with_name("serve")
                        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --datasource=[FILE_OR_GDAL_DS] 'GDAL datasource specification'
                                              --detect-geometry-types=[true|false] 'Detect geometry types when undefined'
                                              --qgs=[FILE] 'QGIS project file'
                                              --loglevel=[error|warn|info|debug|trace] 'Log level (Default: info)'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'
                                              --no-transform=[true|false] 'Do not transform to grid SRS'
                                              --cache=[DIR] 'Use tile cache in DIR'
                                              -c, --config=[FILE] 'Load from custom config file'
                                              --bind=[IPADDRESS] 'Bind web server to this address (0.0.0.0 for all)'
                                              --port=[PORT] 'Bind web server to this port'
                                              --openbrowser=[true|false] 'Open backend URL in browser'")
                        .about("Start web server and serve MVT vector tiles"))
        .subcommand(SubCommand::with_name("genconfig")
                        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --datasource=[FILE_OR_GDAL_DS] 'GDAL datasource specification'
                                              --detect-geometry-types=[true|false] 'Detect geometry types when undefined'
                                              --qgs=[FILE] 'QGIS project file'
                                              --loglevel=[error|warn|info|debug|trace] 'Log level (Default: info)'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'
                                              --no-transform=[true|false] 'Do not transform to grid SRS'")
                        .about("Generate configuration template"))
        .subcommand(SubCommand::with_name("generate")
                        .setting(AppSettings::AllowLeadingHyphen)
                        .args_from_usage("-c, --config=<FILE> 'Load from custom config file'
                                              --loglevel=[error|warn|info|debug|trace] 'Log level (Default: info)'
                                              --tileset=[NAME] 'Tileset name'
                                              --minzoom=[LEVEL] 'Minimum zoom level'
                                              --maxzoom=[LEVEL] 'Maximum zoom level'
                                              --extent=[minx,miny,maxx,maxy[,srid]] 'Extent of tiles'
                                              --nodes=[NUM] 'Number of generator nodes'
                                              --nodeno=[NUM] 'Number of this nodes (0 <= n < nodes)'
                                              --progress=[true|false] 'Show progress bar'
                                              --overwrite=[false|true] 'Overwrite previously cached tiles'")
                        .about("Generate tiles for cache"))
        .subcommand(SubCommand::with_name("drilldown")
                        .setting(AppSettings::AllowLeadingHyphen)
                        .args_from_usage("-c, --config=<FILE> 'Load from custom config file'
                                              --loglevel=[error|warn|info|debug|trace] 'Log level (Default: info)'
                                              --tileset=[NAME] 'Tileset name'
                                              --minzoom=[LEVEL] 'Minimum zoom level'
                                              --maxzoom=[LEVEL] 'Maximum zoom level'
                                              --points=[x1,y1,x2,y2,..] 'Drilldown points'
                                              --progress=[true|false] 'Show progress bar'")
                        .about("Tile layer statistics"));

    match app.get_matches_from_safe_borrow(env::args()) {
        //app.get_matches() prohibits later call of app.print_help()
        Result::Err(e) => {
            println!("{}", e);
        }
        Result::Ok(matches) => match matches.subcommand() {
            ("serve", Some(sub_m)) => {
                init_logger(sub_m);
                let _ = webserver::webserver(sub_m.clone());
            }
            ("genconfig", Some(sub_m)) => {
                init_logger(sub_m);
                println!("{}", webserver::gen_config(sub_m));
            }
            ("generate", Some(sub_m)) => {
                init_logger(sub_m);
                generate(sub_m);
            }
            ("drilldown", Some(sub_m)) => {
                init_logger(sub_m);
                drilldown(sub_m);
            }
            _ => {
                let _ = app.print_help();
                println!("");
            }
        },
    }
}
