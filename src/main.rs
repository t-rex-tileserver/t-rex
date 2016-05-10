#[macro_use] extern crate nickel;
extern crate hyper;

extern crate postgres;
extern crate postgis;
extern crate protobuf;

mod core;
mod datasource;
mod mvt;
mod service;
mod webserver;


fn main() {
    webserver::server::webserver();
}
