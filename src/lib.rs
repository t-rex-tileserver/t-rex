//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

#[macro_use]
extern crate log;
extern crate toml;
extern crate postgres;
extern crate postgis;
extern crate protobuf;
#[macro_use] extern crate nickel;
extern crate nickel_mustache;
extern crate rustc_serialize;
extern crate hyper;
extern crate clap;

mod core;
mod datasource;
mod mvt;
mod service;
mod webserver;
mod config;
