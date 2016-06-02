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
use core::{Config,read_config};
use cache::{Cache,Tilecache,Nocache,Filecache};

use nickel::{Nickel, Options, HttpRouter, MediaType, Request, Responder, Response, MiddlewareResult };
use nickel_mustache::Render;
use hyper::header::{CacheControl, CacheDirective, AccessControlAllowOrigin, AccessControlAllowMethods};
use hyper::method::Method;
use hyper::header;
use std::collections::HashMap;
use clap::ArgMatches;
use std::process;


fn log_request<'mw>(req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    info!("{} {}", req.origin.method, req.origin.uri);
    res.next_middleware()
}

fn enable_cors<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
  // access-control-allow-methods: GET
  // access-control-allow-origin: *
  res.set(AccessControlAllowMethods(vec![Method::Get]));
  res.set(AccessControlAllowOrigin::Any);

  res.next_middleware()
}

header! { (ContentType, "Content-Type") => [String] }

impl<D> Responder<D> for vector_tile::Tile {
    fn respond<'a>(self, mut res: Response<'a, D>) -> MiddlewareResult<'a, D> {
        res.set_header_fallback(|| ContentType("application/x-protobuf".to_owned()));
        res.set_header_fallback(|| CacheControl(vec![CacheDirective::MaxAge(43200u32)])); //TODO: from cache settings

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

fn service_from_args(args: &ArgMatches) -> MvtService {
    if let Some(cfgpath) = args.value_of("config") {
        info!("Reading configuration from '{}'", cfgpath);
        read_config(cfgpath)
            .and_then(|config| MvtService::from_config(&config))
            .unwrap_or_else(|err| {
                println!("Error reading configuration - {} ", err);
                process::exit(1)
            })
    } else {
        let cache = match args.value_of("cache") {
            None => Tilecache::Nocache(Nocache),
            Some(dir) => Tilecache::Filecache(Filecache { basepath: dir.to_string() })
        };
        if let Some(dbconn) = args.value_of("dbconn") {
            let pg = PostgisInput { connection_url: dbconn.to_string() };
            let grid = Grid::web_mercator();
            let layers = pg.detect_layers();
            MvtService {input: pg, grid: grid, layers: layers,
                topics: Vec::new(), cache: cache}
        } else {
            println!("Either 'config' or 'dbconn' is required");
            process::exit(1)
        }
    }
}

pub fn webserver(args: &ArgMatches) {
    let mut server = Nickel::new();
    server.options = Options::default()
                     .thread_count(Some(1));
    server.utilize(log_request);
    server.utilize(enable_cors);

    let service = service_from_args(args);

    let mut layers_display: Vec<LayerInfo> = service.layers.iter().map(|l| {
        LayerInfo::from_layer(l)
    }).collect();
    layers_display.sort_by_key(|li| li.name.clone());

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

pub fn gen_config(args: &ArgMatches) -> String {
        let toml = r#"
[webserver]
# Bind address. Use 0.0.0.0 to listen on all adresses.
bind = "127.0.0.1"
port = 6767
threads = 1
mapviewer = true
"#;
    let mut config = String::new();
    if let Some(dbconn) = args.value_of("dbconn") {
        let service = service_from_args(args);
        config = service.gen_runtime_config();
    } else {
        config = MvtService::gen_config();
    }
    config.push_str(toml);
    config
}


#[test]
fn test_gen_config() {
    use core::parse_config;

    let args = ArgMatches::new();
    let toml = gen_config(&args);
    println!("{}", toml);
    assert_eq!(Some("# t-rex configuration"), toml.lines().next());

    let config = parse_config(toml, "").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    assert_eq!(service.input.connection_url, "postgresql://user:pass@host:port/database");
}

#[test]
fn test_runtime_config() {
    use std::io::{self,Write};
    use std::env;
    use clap::App;
    use core::parse_config;

    if env::var("DBCONN").is_err() {
        write!(&mut io::stdout(), "skipped ").unwrap();
        return;
    }
    let args = App::new("test")
                .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'")
                .get_matches_from(vec!["", "--dbconn", &env::var("DBCONN").unwrap()]);
    let toml = gen_config(&args);
    println!("{}", toml);
    assert_eq!(Some("# t-rex configuration"), toml.lines().next());

    let config = parse_config(toml, "").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    assert_eq!(service.input.connection_url, env::var("DBCONN").unwrap());
}
