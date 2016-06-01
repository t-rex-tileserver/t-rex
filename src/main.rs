//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate toml;
#[macro_use] extern crate nickel;
extern crate nickel_mustache;
extern crate rustc_serialize;
#[macro_use] extern crate hyper;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate clap;
extern crate time;

mod core;
mod datasource;
mod mvt;
mod service;
mod cache;
mod webserver;

use clap::{App, SubCommand};
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
            record.args()
        )
    };

    let mut builder = LogBuilder::new();
    builder.format(format);

    match env::var("RUST_LOG") {
        Result::Ok(val) => { builder.parse(&val); },
        // Set log level for webserver to info by default
        Result::Err(_) => { builder.filter(None, LogLevelFilter::Error).filter(Some("t_rex::webserver::server"), LogLevelFilter::Info); }
    }

    builder.init().unwrap();
}

fn main() {
    init_logger();

    // http://kbknapp.github.io/clap-rs/clap/
    let mut app = App::new("t_rex")
                        .version("0.0.0")
                        .author("Pirmin Kalberer <pka@sourcepole.ch>")
                        .about("vector tile server specialized on publishing MVT tiles from a PostGIS database")
                        .subcommand(SubCommand::with_name("serve")
                            .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'
                                              -c, --config=[FILE] 'Load from custom config file'")
                            .about("Start web server and serve MVT vector tiles"))
                        .subcommand(SubCommand::with_name("genconfig")
                            .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'")
                            .about("Generate configuration template"));
    let matches = app.get_matches_from_safe_borrow(env::args()).unwrap(); //app.get_matches() prohibits later call of app.print_help()
    match matches.subcommand() {
        ("serve", Some(sub_m))     => webserver::server::webserver(sub_m),
        ("genconfig", Some(sub_m)) => println!("{}", webserver::server::gen_config(sub_m)),
        _                          => { app.print_help(); },
    }
}
