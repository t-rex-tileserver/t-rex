use datasource::postgis::PostgisInput;
use core::grid::Grid;
use core::layer::Layer;
use mvt::tile::Tile;
use mvt::vector_tile;
use service::mvt::Topic;
use service::mvt::MvtService;

use nickel::{Nickel, HttpRouter, MediaType, Responder, Response, MiddlewareResult };
use hyper::header;
use std::collections::HashMap;


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

pub fn webserver() {
    let mut server = Nickel::new();

    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let grid = Grid::web_mercator();
    let layers = pg.detect_layers();
    let service = MvtService {input: pg, grid: grid, layers: layers, topics: Vec::new()};

    server.get("/:topic/:z/:x/:y.pbf", middleware! { |req|
        let topic = req.param("topic").unwrap();
        let z = req.param("z").unwrap().parse::<u16>().unwrap();
        let x = req.param("x").unwrap().parse::<u16>().unwrap();
        let y = req.param("y").unwrap().parse::<u16>().unwrap();

        let mvt_tile = service.tile(topic, x, y, z);

        mvt_tile
    });
    server.get("/:topic/", middleware! { |req, res|
        let topic = req.param("topic").unwrap();
        let mut data = HashMap::<&str, &str>::new();
        data.insert("baseurl", "http://127.0.0.1:6767");
        data.insert("topic", topic);
        return res.render("src/webserver/templates/olviewer.tpl", &data)
    });
    server.listen("127.0.0.1:6767");
}
