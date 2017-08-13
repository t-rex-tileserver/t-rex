//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

extern crate log;
extern crate env_logger;
#[macro_use]
extern crate clap;
extern crate time;

extern crate t_rex_core;
extern crate t_rex_webserver;

use t_rex_core::core::grid::Extent;
use t_rex_webserver as webserver;
use clap::{App, SubCommand, ArgMatches, AppSettings};
use std::env;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;


fn init_logger() {
    let format = |record: &LogRecord| {
        let t = time::now();
        format!("{}.{:03} {} {}",
                time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
                t.tm_nsec / 1000_000,
                record.level(),
                record.args())
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    match env::var("RUST_LOG") {
        Result::Ok(val) => {
            builder.parse(&val);
        }
        // Set log level to info by default
        Result::Err(_) => {
            builder.filter(None, LogLevelFilter::Info);
        }
    }

    builder.init().unwrap();
}

fn generate(args: &ArgMatches) {
    let (mut service, config) = webserver::server::service_from_args(args);
    config
        .cache
        .expect("Missing configuration entry base in [cache.file]");
    let tileset = args.value_of("tileset");
    let minzoom = args.value_of("minzoom")
        .map(|s| {
                 s.parse::<u8>()
                     .expect("Error parsing 'minzoom' as integer value")
             });
    let maxzoom = args.value_of("maxzoom")
        .map(|s| {
                 s.parse::<u8>()
                     .expect("Error parsing 'maxzoom' as integer value")
             });
    let extent = args.value_of("extent")
        .and_then(|numlist| {
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
    let nodes = args.value_of("nodes")
        .map(|s| {
                 s.parse::<u8>()
                     .expect("Error parsing 'nodes' as integer value")
             });
    let nodeno = args.value_of("nodeno")
        .map(|s| {
                 s.parse::<u8>()
                     .expect("Error parsing 'nodeno' as integer value")
             });
    let progress = args.value_of("progress")
        .map_or(true, |s| {
            s.parse::<bool>()
                .expect("Error parsing 'progress' as boolean value")
        });
    let overwrite = args.value_of("overwrite")
        .map_or(false, |s| {
            s.parse::<bool>()
                .expect("Error parsing 'overwrite' as boolean value")
        });
    service.prepare_feature_queries();
    service.generate(tileset,
                     minzoom,
                     maxzoom,
                     extent,
                     nodes,
                     nodeno,
                     progress,
                     overwrite);
}

fn main() {
    init_logger();

    // http://kbknapp.github.io/clap-rs/clap/
    let mut app = App::new("t_rex")
        .version(crate_version!())
        .author("Pirmin Kalberer <pka@sourcepole.ch>")
        .about("vector tile server specialized on publishing MVT tiles from a PostGIS database")
        .subcommand(SubCommand::with_name("serve")
                        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --datasource=[FILE_OR_GDAL_DS] 'GDAL datasource specification'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'
                                              --cache=[DIR] 'Use tile cache in DIR'
                                              -c, --config=[FILE] 'Load from custom config file'
                                              --openbrowser=[true|false] 'Open backend URL in browser'")
                        .about("Start web server and serve MVT vector tiles"))
        .subcommand(SubCommand::with_name("genconfig")
                        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --datasource=[FILE_OR_GDAL_DS] 'GDAL datasource specification'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'")
                        .about("Generate configuration template"))
        .subcommand(SubCommand::with_name("generate")
                        .setting(AppSettings::AllowLeadingHyphen)
                        .args_from_usage("-c, --config=<FILE> 'Load from custom config file'
                                              --tileset=[NAME] 'Tileset name'
                                              --minzoom=[LEVEL] 'Minimum zoom level'
                                              --maxzoom=[LEVEL] 'Maximum zoom level'
                                              --extent=[minx,miny,maxx,maxy] 'Extent of tiles'
                                              --nodes=[NUM] 'Number of generator nodes'
                                              --nodeno=[NUM] 'Number of this nodes (0 <= n < nodes)'
                                              --progress=[true|false] 'Show progress bar'
                                              --overwrite=[false|true] 'Overwrite previously cached tiles'")
                        .about("Generate tiles for cache"));

    match app.get_matches_from_safe_borrow(env::args()) { //app.get_matches() prohibits later call of app.print_help()
        Result::Err(e) => {
            println!("{}", e);
        }
        Result::Ok(matches) => {
            match matches.subcommand() {
                ("serve", Some(sub_m)) => webserver::server::webserver(sub_m),
                ("genconfig", Some(sub_m)) => println!("{}", webserver::server::gen_config(sub_m)),
                ("generate", Some(sub_m)) => generate(sub_m),
                _ => {
                    let _ = app.print_help();
                    println!("");
                }
            }
        }
    }
}
