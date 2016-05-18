//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::postgis::PostgisInput;
use core::grid::Grid;
use mvt::tile::Tile;
use mvt::vector_tile;
use service::mvt::MvtService;
use core::layer::Layer;

use nickel::{Nickel, Options, HttpRouter, MediaType, Request, Responder, Response, MiddlewareResult };
use nickel_mustache::Render;
use hyper::header;
use std::collections::HashMap;
use clap::ArgMatches;

fn log_request<'mw>(req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    info!("{} {}", req.origin.method, req.origin.uri);
    res.next_middleware()
}

fn maybe_set_type<D>(res: &mut Response<D>, mime: MediaType) {
    res.set_header_fallback(|| header::ContentType(mime.into()));
}

impl<D> Responder<D> for vector_tile::Tile {
    fn respond<'a>(self, mut res: Response<'a, D>) -> MiddlewareResult<'a, D> {
        maybe_set_type(&mut res, MediaType::Bin);
        let vec = Tile::binary_tile(&self);
        res.send(vec)
    }
}

#[derive(RustcEncodable)]
struct LayerInfo {
    name: String,
    geomtype: String,
    hasviewer: bool,
}

impl LayerInfo {
    fn from_layer(l: &Layer) -> LayerInfo {
        LayerInfo {
            name: l.name.clone(),
            geomtype: l.geometry_type.as_ref().unwrap().clone(),
            hasviewer: (["POINT","LINESTRING","POLYGON"].contains(
                &(l.geometry_type.as_ref().unwrap() as &str)))
        }
    }
}

pub fn webserver(args: &ArgMatches) {
    let mut server = Nickel::new();
    server.options = Options::default()
                     .thread_count(Some(1));
    server.utilize(log_request);

    let dbconn = args.value_of("dbconn").unwrap();
    let pg = PostgisInput { connection_url: dbconn.to_string() };
    let grid = Grid::web_mercator();
    let layers = pg.detect_layers();
    let mut layers_display: Vec<LayerInfo> = layers.iter().map(|l| {
        LayerInfo::from_layer(l)
    }).collect();
    layers_display.sort_by_key(|li| li.name.clone());
    let service = MvtService {input: pg, grid: grid, layers: layers, topics: Vec::new()};

    server.get("/:topic/:z/:x/:y.pbf", middleware! { |req|
        let topic = req.param("topic").unwrap();
        let z = req.param("z").unwrap().parse::<u16>().unwrap();
        let x = req.param("x").unwrap().parse::<u16>().unwrap();
        let y = req.param("y").unwrap().parse::<u16>().unwrap();

        let mvt_tile = service.tile(topic, x, y, z);

        mvt_tile
    });
    server.get("/", middleware! { |req, res|
        let mut data = HashMap::new();
        data.insert("layer", &layers_display);
        return res.render("src/webserver/templates/index.tpl", &data)
    });
    server.get("/:topic/", middleware! { |req, res|
        let topic = req.param("topic").unwrap();
        let host = req.origin.headers.get::<header::Host>().unwrap();
        let baseurl = format!("http://{}:{}", host.hostname, host.port.unwrap_or(80));
        let mut data = HashMap::new();
        data.insert("baseurl", baseurl);
        data.insert("topic", topic.to_string());
        return res.render("src/webserver/templates/olviewer.tpl", &data)
    });
    server.listen("127.0.0.1:6767");
}
