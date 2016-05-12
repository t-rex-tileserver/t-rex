#[macro_use] extern crate nickel;
extern crate nickel_mustache;
extern crate hyper;

extern crate postgres;
extern crate postgis;
extern crate protobuf;
extern crate clap;

mod core;
mod datasource;
mod mvt;
mod service;
mod webserver;

use clap::{App, SubCommand};


fn main() {
    // http://kbknapp.github.io/clap-rs/clap/
    let matches = App::new("t_rex")
                        .version("0.0.0")
                        .author("Pirmin Kalberer <pka@sourcepole.ch>")
                        .about("vector tile server specialized on publishing MVT tiles from a PostGIS database")
                        .subcommand(SubCommand::with_name("serve")
                            .args_from_usage("--dbconn=<SPEC> 'PostGIS connection postgresql://USER@HOST/DBNAME'")
                            .about("Start web server and serve MVT vector tiles"))
                        .get_matches();

     if let Some(ref matches) = matches.subcommand_matches("serve") {
        webserver::server::webserver(matches)
     }
}
