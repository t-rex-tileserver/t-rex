//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::postgis::PostgisInput;
use core::grid::Grid;
use mvt::tile::Tile;
use mvt::vector_tile;
use service::mvt::MvtService;

use nickel::{Nickel, HttpRouter, MediaType, Responder, Response, MiddlewareResult };
use nickel_mustache::Render;
use hyper::header;
use std::collections::HashMap;
use clap::ArgMatches;


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

pub fn webserver(args: &ArgMatches) {
    let mut server = Nickel::new();
    let dbconn = args.value_of("dbconn").unwrap();
    let pg = PostgisInput { connection_url: dbconn.to_string() };
    let grid = Grid::web_mercator();
    let layers = pg.detect_layers();
    let layers_display: Vec<HashMap<&str,String>> = layers.iter().map(|l| {
        let mut h = HashMap::new();
        h.insert("name", l.name.clone());
        h.insert("geomtype", l.geometry_type.as_ref().unwrap().clone());
        h
    }).collect();
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
        let mut data = HashMap::new();
        data.insert("baseurl", "http://127.0.0.1:6767");
        data.insert("topic", topic);
        return res.render("src/webserver/templates/olviewer.tpl", &data)
    });
    server.listen("127.0.0.1:6767");
}
