#[macro_use] extern crate nickel;
extern crate hyper;

extern crate postgres;
extern crate postgis;
extern crate protobuf;

mod core;
mod datasource;
mod mvt;
mod service;

use datasource::postgis::PostgisInput;
use core::grid::Grid;
use core::layer::Layer;
use mvt::tile::Tile;
use service::mvt::Topic;
use service::mvt::MvtService;

use nickel::{Nickel, HttpRouter, MediaType, Responder, Response, MiddlewareResult };
use hyper::header;


fn maybe_set_type<D>(res: &mut Response<D>, mime: MediaType) {
    res.set_header_fallback(|| header::ContentType(mime.into()));
}

impl<D> Responder<D> for mvt::vector_tile::Tile {
    fn respond<'a>(self, mut res: Response<'a, D>) -> MiddlewareResult<'a, D> {
        maybe_set_type(&mut res, MediaType::Bin);
        let vec = Tile::binary_tile(&self);
        res.send(vec)
    }
}

fn main() {
    let mut server = Nickel::new();

    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let grid = Grid::web_mercator();
    let layers = vec![Layer {
        name: String::from("points"),
        query: String::from("SELECT geometry FROM osm_place_point")
    }];
    let topics = vec![Topic {name: String::from("roads"), layers: layers}];
    let service = MvtService {input: pg, grid: grid, topics: topics};

    server.get("/:topic/:z/:x/:y.pbf", middleware! { |req|
        let topic = req.param("topic").unwrap();
        let z = req.param("z").unwrap().parse::<u16>().unwrap();
        let x = req.param("x").unwrap().parse::<u16>().unwrap();
        let y = req.param("y").unwrap().parse::<u16>().unwrap();

        let mvt_tile = service.tile(topic, x, y, z);

        mvt_tile
    });

    server.listen("127.0.0.1:6767");
}
