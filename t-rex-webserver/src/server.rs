//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use cache::{Filecache, Nocache, Tilecache};
use core::config::ApplicationCfg;
use core::config::DEFAULT_CONFIG;
use core::grid::Grid;
use core::layer::Layer;
use core::{parse_config, read_config, Config};
use datasource::DatasourceInput;
use datasource_type::Datasources;
use mvt_service::MvtService;
use read_qgs;
use service::tileset::Tileset;

use actix;
use actix_web::{fs, http::header, http::ContentEncoding, http::Method, middleware,
                middleware::cors::Cors, server::HttpServer, App, Error, HttpRequest, HttpResponse,
                Path};
use clap::ArgMatches;
use futures::future::{result, FutureResult};
use open;
use std::collections::HashMap;
use std::process;
use std::str;
use std::str::FromStr;

struct StaticFiles {
    files: HashMap<&'static str, (&'static [u8], &'static str)>,
}

impl StaticFiles {
    fn init() -> StaticFiles {
        let mut static_files = StaticFiles {
            files: HashMap::new(),
        };
        static_files.add(
            "favicon.ico",
            include_bytes!("static/favicon.ico"),
            "image/x-icon",
        );
        static_files.add(
            "index.html",
            include_bytes!("static/index.html"),
            "text/html",
        );
        static_files.add(
            "viewer.js",
            include_bytes!("static/viewer.js"),
            "application/javascript",
        );
        static_files.add(
            "viewer.css",
            include_bytes!("static/viewer.css"),
            "text/css",
        );
        static_files.add(
            "maputnik.html",
            include_bytes!("static/maputnik.html"),
            "text/html",
        );
        static_files.add(
            "maputnik.js",
            include_bytes!("static/maputnik.js"),
            "application/javascript",
        );
        static_files.add(
            "maputnik-vendor.js",
            include_bytes!("static/maputnik-vendor.js"),
            "application/javascript",
        );
        static_files.add(
            "img/maputnik.png",
            include_bytes!("static/img/maputnik.png"),
            "image/png",
        );
        static_files.add(
            "fonts/Roboto-Regular.ttf",
            include_bytes!("static/fonts/Roboto-Regular.ttf"),
            "font/ttf",
        );
        static_files.add(
            "fonts/Roboto-Medium.ttf",
            include_bytes!("static/fonts/Roboto-Medium.ttf"),
            "font/ttf",
        );
        static_files
    }
    fn add(&mut self, name: &'static str, data: &'static [u8], media_type: &'static str) {
        self.files.insert(name, (data, media_type));
    }
    fn content(&self, base: Option<&str>, name: String) -> Option<&(&[u8], &str)> {
        let mut key = if name == "" {
            "index.html".to_string()
        } else {
            name
        };
        if let Some(path) = base {
            key = format!("{}/{}", path, key);
        }
        self.files.get(&key as &str)
    }
}

lazy_static! {
    static ref STATIC_FILES: StaticFiles = StaticFiles::init();
}

static DINO: &'static str = "             xxxxxxxxx
        xxxxxxxxxxxxxxxxxxxxxxxx
      xxxxxxxxxxxxxxxxxxxxxxxxxxxx
     xxxxxxxxxxxxxxxxxxxxxxxxx xxxx
     xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
   xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
 xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx xxxxxxxxxxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx  xxxxxxxxxxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx   xxxxxxxxxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx   xxxxxxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx     xxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx      x
xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
xxxxxxxxxxxxxxxxxxxxxxxxxx    xxxxxxxxxxx
xxxxxxxxxxxxxx                   xxxxxx
xxxxxxxxxxxx
xxxxxxxxxxx
xxxxxxxxxx
xxxxxxxxx
xxxxxxx
xxxxxx
xxxxxxx";

fn set_layer_buffer_defaults(layer: &mut Layer, simplify: bool, clip: bool) {
    layer.simplify = simplify;
    if simplify {
        // Limit features by default unless simplify is set to false
        layer.query_limit = Some(1000);
    }
    layer.buffer_size = match layer.geometry_type {
        Some(ref geom) => {
            if clip {
                if geom.contains("POLYGON") {
                    Some(1)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        }
        None => None,
    };
}

pub fn config_from_args(args: &ArgMatches) -> ApplicationCfg {
    if let Some(cfgpath) = args.value_of("config") {
        info!("Reading configuration from '{}'", cfgpath);
        for argname in vec!["dbconn", "datasource", "qgs"] {
            if args.value_of(argname).is_some() {
                warn!("Ignoring argument `{}`", argname);
            }
        }
        let config = read_config(cfgpath).unwrap_or_else(|err| {
            println!("Error reading configuration - {} ", err);
            process::exit(1)
        });
        config
    } else {
        let bind = args.value_of("bind").unwrap_or("127.0.0.1");
        let port =
            u16::from_str(args.value_of("port").unwrap_or("6767")).expect("Invalid port number");
        let mut config: ApplicationCfg = parse_config(DEFAULT_CONFIG.to_string(), "").unwrap();
        config.webserver.bind = Some(bind.to_string());
        config.webserver.port = Some(port);
        config
    }
}

pub fn service_from_args(config: &ApplicationCfg, args: &ArgMatches) -> MvtService {
    if args.value_of("config").is_some() {
        let mut svc = MvtService::from_config(&config).unwrap_or_else(|err| {
            println!("Error reading configuration - {} ", err);
            process::exit(1)
        });
        svc.connect();
        svc
    } else {
        let cache = match args.value_of("cache") {
            None => Tilecache::Nocache(Nocache),
            Some(dir) => Tilecache::Filecache(Filecache {
                basepath: dir.to_string(),
                baseurl: None,
            }),
        };
        let simplify = bool::from_str(args.value_of("simplify").unwrap_or("true")).unwrap_or(false);
        let clip = bool::from_str(args.value_of("clip").unwrap_or("true")).unwrap_or(false);
        let grid = Grid::web_mercator();
        let mut tilesets = Vec::new();
        let datasources = if let Some(qgs) = args.value_of("qgs") {
            info!("Reading configuration from '{}'", qgs);
            let (datasources, mut tileset) = read_qgs(qgs);
            for layer in tileset.layers.iter_mut() {
                set_layer_buffer_defaults(layer, simplify, clip);
            }
            tilesets.push(tileset);
            datasources
        } else {
            let datasources = Datasources::from_args(args);
            if datasources.datasources.is_empty() {
                println!("Either 'config', 'dbconn' or 'datasource' is required");
                process::exit(1)
            }
            let detect_geometry_types = true; //TODO: add option (maybe slow for many geometries)
            for (_name, ds) in &datasources.datasources {
                let dsconn = ds.connected();
                let mut layers = dsconn.detect_layers(detect_geometry_types);
                while let Some(mut l) = layers.pop() {
                    let extent = dsconn.layer_extent(&l);
                    set_layer_buffer_defaults(&mut l, simplify, clip);
                    let tileset = Tileset {
                        name: l.name.clone(),
                        minzoom: None,
                        maxzoom: None,
                        attribution: None,
                        extent: extent,
                        center: None,
                        start_zoom: None,
                        layers: vec![l],
                    };
                    tilesets.push(tileset);
                }
            }
            datasources
        };
        let mut svc = MvtService {
            datasources: datasources,
            grid: grid,
            tilesets: tilesets,
            cache: cache,
        };
        svc.connect(); //TODO: ugly - we connect twice
        svc
    }
}

/// Application state
struct AppState {
    service: MvtService,
}

fn mvt_metadata(req: HttpRequest<AppState>) -> FutureResult<HttpResponse, Error> {
    let json = req.state().service.get_mvt_metadata().unwrap();
    result(Ok(HttpResponse::Ok().json(json)))
}

/// Font list for Maputnik
fn fontstacks(_req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(["Roboto Medium", "Roboto Regular"]))
}

// Include method fonts() which returns HashMap with embedded font files
include!(concat!(env!("OUT_DIR"), "/fonts.rs"));

/// Fonts for Maputnik
/// Example: /fonts/Open%20Sans%20Regular,Arial%20Unicode%20MS%20Regular/0-255.pbf
fn fonts_pbf(
    (_req, params): (HttpRequest<AppState>, Path<(String, String)>),
) -> Result<HttpResponse, Error> {
    let fontpbfs = fonts();
    let fontlist = &params.0;
    let range = &params.1;
    let mut fonts = fontlist.split(",").collect::<Vec<_>>();
    fonts.push("Roboto Regular"); // Fallback
    let mut resp = HttpResponse::NotFound().finish();
    for font in fonts {
        let key = format!("fonts/{}/{}.pbf", font.replace("%20", " "), range);
        debug!("Font lookup: {}", key);
        if let Some(pbf) = fontpbfs.get(&key as &str) {
            resp = HttpResponse::Ok()
                .content_type("application/x-protobuf")
                // data is already gzip compressed
                .content_encoding(ContentEncoding::Identity)
                .header(header::CONTENT_ENCODING, "gzip")
                .body(*pbf); // TODO: chunked response
            break;
        }
    }
    Ok(resp)
}

fn req_baseurl(req: &HttpRequest<AppState>) -> String {
    let conninfo = req.connection_info();
    format!("{}://{}", conninfo.scheme(), conninfo.host())
}

fn tileset_tilejson(
    (req, tileset): (HttpRequest<AppState>, Path<String>),
) -> FutureResult<HttpResponse, Error> {
    let json = req.state()
        .service
        .get_tilejson(&req_baseurl(&req), &tileset)
        .unwrap();
    result(Ok(HttpResponse::Ok().json(json)))
}

fn tileset_style_json(
    (req, tileset): (HttpRequest<AppState>, Path<String>),
) -> FutureResult<HttpResponse, Error> {
    let json = req.state()
        .service
        .get_stylejson(&req_baseurl(&req), &tileset)
        .unwrap();
    result(Ok(HttpResponse::Ok().json(json)))
}

fn tileset_metadata_json(
    (req, tileset): (HttpRequest<AppState>, Path<String>),
) -> FutureResult<HttpResponse, Error> {
    let json = req.state().service.get_mbtiles_metadata(&tileset).unwrap();
    result(Ok(HttpResponse::Ok().json(json)))
}

fn tile_pbf(
    (req, params): (HttpRequest<AppState>, Path<(String, u8, u32, u32)>),
) -> FutureResult<HttpResponse, Error> {
    let tileset = &params.0;
    let z = params.1;
    let x = params.2;
    let y = params.3;
    let gzip = true;
    /* TODO:
    let gzip = accept_encoding.is_some() && accept_encoding.unwrap().iter().any(
               |ref qit| qit.item == Encoding::Gzip );
               */
    let tile = req.state().service.tile_cached(tileset, x, y, z, gzip);
    let resp = HttpResponse::Ok()
        .content_type("application/x-protobuf")
        .if_true(gzip, |r| {
            // data is already gzip compressed
            r.content_encoding(ContentEncoding::Identity)
                .header(header::CONTENT_ENCODING, "gzip");
        })
        .body(tile); // TODO: chunked response
    /* TODO:
        res.set_header_fallback(|| CacheControl(vec![CacheDirective::MaxAge(cache_max_age)]));
    */
    result(Ok(resp))
}

fn static_file_handler(req: HttpRequest<AppState>) -> Result<HttpResponse, Error> {
    let key = req.path()[1..].to_string();
    let resp = if let Some(ref content) = STATIC_FILES.content(None, key) {
        HttpResponse::Ok().content_type(content.1).body(content.0) // TODO: chunked response
    } else {
        HttpResponse::NotFound().finish()
    };
    Ok(resp)
}

pub fn webserver(args: ArgMatches<'static>) {
    let config = config_from_args(&args);
    let host = config.webserver.bind.unwrap_or("127.0.0.1".to_string());
    let port = config.webserver.port.unwrap_or(6767);
    let bind_addr = format!("{}:{}", host, port);
    let mvt_viewer = config.service.mvt.viewer;
    let openbrowser =
        bool::from_str(args.value_of("openbrowser").unwrap_or("true")).unwrap_or(false);
    let _threads = config.webserver.threads.unwrap_or(4) as usize; //TODO (?)

    actix::System::run(move || {
        HttpServer::new(move || {
            let config = config_from_args(&args);
            let mut service = service_from_args(&config, &args);

            let mvt_viewer = config.service.mvt.viewer;
            let _cache_max_age = config.webserver.cache_control_max_age.unwrap_or(300);

            service.prepare_feature_queries();
            service.init_cache();

            App::with_state(AppState{service: service})
                .middleware(middleware::Logger::default())
                .resource("/index.json", |r| r.method(Method::GET).a(mvt_metadata))
                /* TODO: CORS does only set allowed_origin. actix-web bug?
                .configure(|app| {
                    Cors::for_app(app)
                        .allowed_origin("*")
                        .allowed_methods(vec!["GET"])
                        .allowed_headers(vec![header::ACCEPT])
                        .resource("/fontstacks.json", |r| r.method(Method::GET).f(fontstacks))
                        .resource("/fonts/{fonts}/{range}.pbf", |r| r.method(Method::GET).with(fonts_pbf))
                        .resource("/{tileset}.style.json", |r| r.method(Method::GET).with_async(tileset_style_json))
                        .resource("/{tileset}/metadata.json", |r| r.method(Method::GET).with_async(tileset_metadata_json))
                        .resource("/{tileset}.json", |r| r.method(Method::GET).with_async(tileset_tilejson))
                        .resource("/{tileset}/{z}/{x}/{y}.pbf", |r| r.method(Method::GET).with_async(tile_pbf))
                        .register()
                });*/
                .resource("/fontstacks.json", |r| r.method(Method::GET).f(fontstacks))
                .resource("/fonts/{fonts}/{range}.pbf", |r| r.method(Method::GET).with(fonts_pbf))
                .resource("/{tileset}.style.json", |r| r.method(Method::GET).with_async(tileset_style_json))
                .resource("/{tileset}/metadata.json", |r| r.method(Method::GET).with_async(tileset_metadata_json))
                .resource("/{tileset}.json", |r| r.method(Method::GET).with_async(tileset_tilejson))
                .resource("/{tileset}/{z}/{x}/{y}.pbf", |r| r.method(Method::GET).with_async(tile_pbf))
                .configure(|app| {
                    /*
                    if ./public/ exists {
                        app.handler(
                            "/",
                            fs::StaticFiles::new("./public/")
                        )
                    }*/
                    if mvt_viewer {
                        app.handler("/", static_file_handler)
                    } else {
                        app
                    }
                })
        }).bind(&bind_addr)
            .expect("Can not start server on given IP/Port")
            .shutdown_timeout(3) // default: 30s
            .start();

        println!("{}", DINO);

        if openbrowser && mvt_viewer {
            let _res = open::that(format!("http://{}:{}", &host, port));
        }
    });
}

pub fn gen_config(args: &ArgMatches) -> String {
    let toml = r#"
[webserver]
# Bind address. Use 0.0.0.0 to listen on all adresses.
bind = "127.0.0.1"
port = 6767
threads = 4
#cache_control_max_age = 43200
"#;
    let mut config;
    if args.value_of("dbconn").is_some() || args.value_of("datasource").is_some()
        || args.value_of("qgs").is_some()
    {
        let service = service_from_args(&config_from_args(args), args);
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
    let _service = MvtService::from_config(&config).unwrap();
    //assert_eq!(service.input.connection_url,
    //           "postgresql://user:pass@host/database");
}

#[test]
#[ignore]
fn test_runtime_config() {
    use clap::App;
    use core::parse_config;
    use std::env;

    if env::var("DBCONN").is_err() {
        panic!("DBCONN undefined");
    }
    let args = App::new("test")
        .args_from_usage("--dbconn=[SPEC] 'PostGIS connection postgresql://USER@HOST/DBNAME'")
        .get_matches_from(vec!["", "--dbconn", &env::var("DBCONN").unwrap()]);
    let toml = gen_config(&args);
    println!("{}", toml);
    assert_eq!(Some("# t-rex configuration"), toml.lines().next());

    let config = parse_config(toml, "").unwrap();
    let _service = MvtService::from_config(&config).unwrap();
    //assert_eq!(service.input.connection_url, env::var("DBCONN").unwrap());
}
