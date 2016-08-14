//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::postgis::PostgisInput;
use core::grid::Grid;
use mvt::tile::Tile;
use mvt::vector_tile;
use service::mvt::{MvtService,Tileset};
use core::{Config,read_config};
use cache::{Tilecache,Nocache,Filecache};

use nickel::{Nickel, Options, HttpRouter, MediaType, Request, Responder, Response, MiddlewareResult, Halt, StaticFilesHandler};
use mustache;
use rustc_serialize::Encodable;
use hyper::header::{CacheControl, CacheDirective, AccessControlAllowOrigin, AccessControlAllowMethods, ContentEncoding, Encoding};
use hyper::method::Method;
use hyper::header;
use std::collections::HashMap;
use clap::ArgMatches;
use std::str;
use std::process;


fn log_request<'mw>(req: &mut Request<MvtService>, res: Response<'mw,MvtService>) -> MiddlewareResult<'mw,MvtService> {
    info!("{} {}", req.origin.method, req.origin.uri);
    res.next_middleware()
}

#[allow(dead_code)]
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
struct TilesetInfo {
    name: String,
    layerinfos: String,
    hasviewer: bool,
}

impl TilesetInfo {
    fn from_tileset(set: &Tileset) -> TilesetInfo {
        let mut hasviewer = true;
        let layerinfos: Vec<String> = set.layers.iter().map(|l| {
                let geom_type = l.geometry_type.clone().unwrap_or("UNKNOWN".to_string());
                hasviewer = hasviewer && ["POINT","LINESTRING","POLYGON"].contains(&(&geom_type as &str));
                format!("{} [{}]", &l.name, &geom_type)
            }).collect();
        TilesetInfo {
            name: set.name.clone(),
            layerinfos: format!("{}", layerinfos.join(", ")),
            hasviewer: hasviewer
        }
    }
}

struct StaticFiles {
    files: HashMap<&'static str, &'static str>,
}

impl StaticFiles {
    fn new() -> StaticFiles {
        let mut static_files = HashMap::new();
        static_files.insert("tile-inspector", str::from_utf8(include_bytes!("static/tile-inspector.html")).unwrap());
        static_files.insert("vector", str::from_utf8(include_bytes!("static/vector.js")).unwrap());
        static_files.insert("xray", str::from_utf8(include_bytes!("static/xray.html")).unwrap());
        StaticFiles { files: static_files }
    }
}

struct InlineTemplate {
    template: mustache::Template,
}

impl InlineTemplate {
    fn new(template: &str) -> InlineTemplate {
        let tpl = mustache::compile_str(template);
        InlineTemplate { template: tpl }
    }
    // extracted from Nickel::Response#render
    fn render<'a, D, T>(&self, res: Response<'a, D>, data: &T)
            -> MiddlewareResult<'a, D> where T: Encodable {
        let mut stream = try!(res.start());
        match self.template.render(&mut stream, data) {
            Ok(()) => Ok(Halt(stream)),
            Err(e) => stream.bail(format!("Problem rendering template: {:?}", e))
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
            let pg = PostgisInput::new(dbconn);
            let grid = Grid::web_mercator();
            let detect_geometry_types = true; //TODO: add option (maybe slow for many geometries)
            let mut layers = pg.detect_layers(detect_geometry_types);
            let mut tilesets = Vec::new();
            while let Some(l) = layers.pop() {
                let tileset = Tileset{name: l.name.clone(), layers: vec![l]};
                tilesets.push(tileset);
            }
            MvtService {input: pg, grid: grid,
                tilesets: tilesets, cache: cache}
        } else {
            println!("Either 'config' or 'dbconn' is required");
            process::exit(1)
        }
    }
}

pub fn webserver(args: &ArgMatches) {
    let service = service_from_args(args);
    service.init_cache();

    let mut tileset_infos: Vec<TilesetInfo> = service.tilesets.iter().map(|set| {
        TilesetInfo::from_tileset(&set)
    }).collect();
    tileset_infos.sort_by_key(|ti| ti.name.clone());

    let mut server = Nickel::with_data(service);
    server.options = Options::default()
                     .thread_count(Some(1));
    server.utilize(log_request);

    server.get("/:tileset.json", middleware! { |req, mut res|
        let service: &MvtService = res.server_data();
        let tileset = req.param("tileset").unwrap();
        res.set(MediaType::Json);
        service.get_tilejson(&tileset)
    });

    server.get("/:tileset/metadata.json", middleware! { |req, mut res|
        let service: &MvtService = res.server_data();
        let tileset = req.param("tileset").unwrap();
        res.set(MediaType::Json);
        service.get_metadata(&tileset)
    });

    server.get("/:tileset/:z/:x/:y.pbf", middleware! { |req, mut res|
        let service: &MvtService = res.server_data();

        let tileset = req.param("tileset").unwrap();
        let z = req.param("z").unwrap().parse::<u8>().unwrap();
        let x = req.param("x").unwrap().parse::<u16>().unwrap();
        let y = req.param("y").unwrap().parse::<u16>().unwrap();

        let gzip = true; // TODO: From AcceptEncoding
        let tile = service.tile_cached(tileset, x, y, z, gzip);
        if gzip {
            res.set_header_fallback(|| ContentEncoding(vec![Encoding::Gzip]));
        }
        res.set_header_fallback(|| ContentType("application/x-protobuf".to_owned()));
        res.set_header_fallback(|| CacheControl(vec![CacheDirective::MaxAge(43200u32)])); //TODO: from cache settings
        //res.set_header_fallback(|| ContentLength(tile.len() as u64));
        res.set(AccessControlAllowMethods(vec![Method::Get]));
        res.set(AccessControlAllowOrigin::Any);

        tile
    });

    let tpl_olviewer = InlineTemplate::new(str::from_utf8(include_bytes!("templates/olviewer.tpl")).unwrap());
    server.get("/:tileset/", middleware! { |req, res|
        let tileset = req.param("tileset").unwrap();
        let host = req.origin.headers.get::<header::Host>().unwrap();
        let baseurl = format!("http://{}:{}", host.hostname, host.port.unwrap_or(80));
        let mut data = HashMap::new();
        data.insert("baseurl", baseurl);
        data.insert("tileset", tileset.to_string());
        return tpl_olviewer.render(res, &data);
    });

    let static_files = StaticFiles::new();
    server.get("/:static", middleware! { |req, res|
        let name = req.param("static").unwrap();
        if let Some(content) = static_files.files.get(name) {
            return res.send(*content)
        }
    });

    server.get("/**", StaticFilesHandler::new("public/"));

    let tpl_index = InlineTemplate::new(str::from_utf8(include_bytes!("templates/index.tpl")).unwrap());
    server.get("/", middleware! { |_req, res|
        let mut data = HashMap::new();
        data.insert("tileset", &tileset_infos);
        return tpl_index.render(res, &data);
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
    let mut config;
    if let Some(_dbconn) = args.value_of("dbconn") {
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

    //let config = parse_config(toml, "").unwrap();
    //MvtService::from_config fails because of invalid port in postgresql://user:pass@host:port/database
    //let service = MvtService::from_config(&config).unwrap();
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
