//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;
#[macro_use] extern crate nickel;
extern crate mustache;
extern crate rustc_serialize;
#[macro_use] extern crate hyper;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate clap;
extern crate time;
extern crate flate2;
extern crate pbr;

pub mod core;
mod datasource;
pub mod mvt;
mod service;
mod cache;
mod webserver;

use core::grid::Extent;
use clap::{App, SubCommand, ArgMatches};
use std::env;
use std::process;
use log::{LogRecord, LogLevelFilter};
use env_logger::LogBuilder;


fn init_logger() {
    let format = |record: &LogRecord| {
        let t = time::now();
        format!("{}.{:03} {} {}",
            time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
            t.tm_nsec / 1000_000,
            record.level(),
            record.args()
        )
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    match env::var("RUST_LOG") {
        Result::Ok(val) => { builder.parse(&val); },
        // Set log level to info by default
        Result::Err(_) => { builder.filter(None, LogLevelFilter::Info); }
    }

    builder.init().unwrap();
}

fn generate(args: &ArgMatches) {
    let (mut service, config) = webserver::server::service_from_args(args);
    let _ = config.lookup("cache.file.base")
        .ok_or("Missing configuration entry base in [cache.file]".to_string())
        .unwrap_or_else(|err| {
            println!("Error reading configuration - {} ", err);
            process::exit(1)
        });
    let tileset = args.value_of("tileset");
    let minzoom = args.value_of("minzoom").map(|s| s.parse::<u8>().unwrap());
    let maxzoom = args.value_of("maxzoom").map(|s| s.parse::<u8>().unwrap());
    let extent = args.values_of("extent").map(|vals| {
        let arr: Vec<f64> = vals.map(|v| v.parse().unwrap()).collect();
        Extent { minx: arr[0], miny: arr[1], maxx: arr[2], maxy: arr[3] }
    });
    let nodes = args.value_of("nodes").map(|s| s.parse::<u8>().unwrap());
    let nodeno = args.value_of("nodeno").map(|s| s.parse::<u8>().unwrap());
    let progress = args.value_of("progress").map_or(true, |s| s.parse::<bool>().unwrap());
    service.prepare_feature_queries();
    service.generate(tileset, minzoom, maxzoom, extent, nodes, nodeno, progress);
}

fn main() {
    init_logger();

    // http://kbknapp.github.io/clap-rs/clap/
    let mut app = App::new("t_rex")
                        .version("0.5.0")
                        .author("Pirmin Kalberer <pka@sourcepole.ch>")
                        .about("vector tile server specialized on publishing MVT tiles from a PostGIS database")
                        .subcommand(SubCommand::with_name("serve")
                            .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'
                                              --cache=[DIR] 'Use tile cache in DIR'
                                              -c, --config=[FILE] 'Load from custom config file'")
                            .about("Start web server and serve MVT vector tiles"))
                        .subcommand(SubCommand::with_name("genconfig")
                            .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              --simplify=[true|false] 'Simplify geometries'
                                              --clip=[true|false] 'Clip geometries'")
                            .about("Generate configuration template"))
                        .subcommand(SubCommand::with_name("generate")
                            .args_from_usage("-c, --config=<FILE> 'Load from custom config file'
                                              --tileset=[NAME] 'Tileset name'
                                              --minzoom=[LEVEL] 'Minimum zoom level'
                                              --maxzoom=[LEVEL] 'Maximum zoom level'
                                              --extent=[minx,miny,maxx,maxy] 'Extent of tiles'
                                              --nodes=[NUM] 'Number of generator nodes'
                                              --nodeno=[NUM] 'Number of this nodes (0 <= n < nodes)'
                                              --progress=[true|false] 'Show progress bar'")
                            .about("Generate tiles for cache"));

    match app.get_matches_from_safe_borrow(env::args()) { //app.get_matches() prohibits later call of app.print_help()
        Result::Err(e) => { println!("{}", e); },
        Result::Ok(matches) => {
            match matches.subcommand() {
                ("serve", Some(sub_m))     => webserver::server::webserver(sub_m),
                ("genconfig", Some(sub_m)) => println!("{}", webserver::server::gen_config(sub_m)),
                ("generate", Some(sub_m))  => generate(sub_m),
                _                          => { let _ = app.print_help(); println!(""); },
            }
        }
    }
}
