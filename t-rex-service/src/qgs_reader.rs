//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::datasources::{Datasource, Datasources};
use elementtree::Element;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use t_rex_core::core::layer::Layer;
#[cfg(test)]
use t_rex_core::core::Config;
#[cfg(not(feature = "with-gdal"))]
use t_rex_core::datasource::DummyDatasource as GdalDatasource;
use t_rex_core::datasource::PostgisDatasource;
use t_rex_core::service::tileset::Tileset;
#[cfg(feature = "with-gdal")]
use t_rex_gdal::{ogr_layer_name, GdalDatasource};

pub fn get_user_name() -> String {
    env::var("LOGNAME").unwrap_or("".to_string())
}

fn read_xml(fname: &str) -> Result<Element, io::Error> {
    let file = File::open(fname)?;
    let mut reader = BufReader::new(file);
    Element::from_reader(&mut reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
}

#[derive(Debug)]
struct PgLayerInfo {
    pub dbconn: String,
    pub geometry_field: String,
    pub geometry_type: String,
    pub srid: i32,
    pub table_name: String,
    pub subquery: Option<String>,
}

impl PgLayerInfo {
    fn from_qgs_ds(ds: &str) -> PgLayerInfo {
        let params: HashMap<&str, &str> = ds
            .split(' ')
            .map(|kv| kv.split('=').collect::<Vec<&str>>())
            .map(|vec| {
                if vec.len() == 2 {
                    (vec[0], vec[1].trim_matches('\''))
                } else {
                    if vec[0].starts_with("(") {
                        ("geometry_field", vec[0].get(1..vec[0].len() - 1).unwrap())
                    } else {
                        (vec[0], "")
                    }
                }
            })
            .collect();

        //postgresql://[user[:password]@][netloc][:port][/dbname][?param1=value1&...]
        let mut uri = "postgresql://".to_string();
        if params.contains_key("user") {
            uri.push_str(params["user"]);
            if params.contains_key("password") {
                uri.push_str(":");
                uri.push_str(params["password"]);
            }
            uri.push_str("@");
        }
        if params.contains_key("host") {
            uri.push_str(params["host"]);
        } else {
            // socket
            if !params.contains_key("user") {
                uri.push_str(&get_user_name());
                uri.push_str("@");
            }
            uri.push_str("%2Frun%2Fpostgresql"); // TODO
        }
        if params.contains_key("port") {
            uri.push_str(":");
            uri.push_str(params["port"]);
        }
        if params.contains_key("dbname") {
            uri.push_str("/");
            uri.push_str(params["dbname"]);
        }

        PgLayerInfo {
            dbconn: uri,
            table_name: params["table"].to_string(),
            geometry_field: params["geometry_field"].to_string(),
            geometry_type: params["type"].to_uppercase(),
            srid: i32::from_str(params["srid"]).unwrap(),
            subquery: None, //TODO
        }
    }
}

#[derive(Debug)]
struct GdalLayerInfo {
    pub path: String,
    pub geometry_field: String,
    pub geometry_type: String,
    pub srid: Option<i32>,
    pub layer_name: String,
    pub subquery: Option<String>,
}

impl GdalLayerInfo {
    fn from_qgs_ds(fname: &str, ds: &str) -> GdalLayerInfo {
        // split ds like "../data/natural_earth.gpkg|layerid=2"
        let parts = ds.split('|').collect::<Vec<&str>>();
        // Resolve path relative to project file path
        let ds_path = Path::new(parts[0]);
        let path = if ds_path.is_absolute() {
            ds_path.to_path_buf()
        } else {
            let qgs_path = Path::new(fname).parent().unwrap_or(Path::new("."));
            qgs_path.join(parts[0]).canonicalize().unwrap()
        };
        // Read layer name based on id
        let layer = parts[1].split('=').collect::<Vec<&str>>();
        let layer_name = match layer[0] {
            "layerid" => ogr_layer_name(path.to_str().unwrap(), isize::from_str(layer[1]).unwrap())
                .expect("Couldn't resolve layer name"),
            "layername" => layer[1].to_string(),
            &_ => format!("<{}>", ds),
        };
        GdalLayerInfo {
            path: path.to_str().unwrap().to_string().replace("//?/", ""),
            geometry_field: "TODO".to_string(),
            geometry_type: "TODO".to_string(),
            srid: None, //TODO
            layer_name: layer_name,
            subquery: None, //TODO
        }
    }
}

#[cfg(not(feature = "with-gdal"))]
pub fn ogr_layer_name(_path: &str, _id: isize) -> Option<String> {
    Some("".to_string())
}

pub fn read_qgs(fname: &str) -> (Datasources, Tileset) {
    let root = read_xml(fname).unwrap();
    let projectlayers = root
        .find("projectlayers")
        .expect("Invalid or empty QGIS Project file");
    let qgs_name = Path::new(fname).file_stem().unwrap().to_str().unwrap();
    let mut datasources = Datasources::new();
    let mut tileset = Tileset {
        name: qgs_name.to_string(),
        minzoom: None,
        maxzoom: None,
        attribution: None,
        extent: None,
        center: None,
        start_zoom: None,
        layers: Vec::new(),
        cache_limits: None,
    };
    for qgslayer in projectlayers.find_all("maplayer") {
        let layertype = qgslayer.get_attr("type").expect("Missing attribute 'type'");
        if layertype != "vector" {
            continue;
        }
        let _minscale = qgslayer.get_attr("minimumScale");
        let _maxscale = qgslayer.get_attr("maximumScale");
        let _geom_type = qgslayer.get_attr("geometry");
        let name = qgslayer
            .find("layername")
            .expect("Missing element 'layername'")
            .text();
        let provider = qgslayer
            .find("provider")
            .expect("Missing element 'provider'")
            .text();
        let dsinfo = qgslayer
            .find("datasource")
            .expect("Missing element 'datasource'")
            .text();
        let mut layer = Layer::new(name);
        let ds = match provider {
            "ogr" => {
                let info = GdalLayerInfo::from_qgs_ds(fname, dsinfo);
                layer.table_name = Some(info.layer_name);
                Datasource::Gdal(GdalDatasource::new(&info.path))
            }
            "postgres" => {
                let info = PgLayerInfo::from_qgs_ds(dsinfo);
                layer.table_name = Some(info.table_name.replace("\"", ""));
                if info.geometry_type != "POINT" {
                    layer.simplify = true;
                    layer.tolerance = "!pixel_width!/2".to_string(); // DEFAULT_TOLERANCE in layer.rs
                }
                layer.geometry_field = Some(info.geometry_field);
                layer.geometry_type = Some(info.geometry_type);
                layer.srid = Some(info.srid);
                Datasource::Postgis(PostgisDatasource::new(&info.dbconn, None, None))
            }
            _ => continue,
        };
        datasources.add(&name.to_string(), ds);
        layer.datasource = Some(name.to_string());
        tileset.layers.push(layer)
    }
    (datasources, tileset)
}

#[test]
fn test_parse_xml() {
    assert!(read_xml("../examples/natural_earth.qgs").is_ok());
    assert!(read_xml("wrong_file_name")
        .err()
        .unwrap()
        .to_string()
        .contains("(os error 2)"));
    // Linux: "No such file or directory (os error 2)"
    assert_eq!(
        &read_xml("Cargo.toml").err().unwrap().to_string(),
        "Malformed XML: Unexpected characters outside the root element: [ (0:0)"
    );
}

#[test]
fn test_pg_uri() {
    let info = PgLayerInfo::from_qgs_ds(
        r#"dbname='natural_earth_vectors' host=localhost port=5432 user='pi' password='xxx' sslmode=allow key='fid' estimatedmetadata=true srid=4326 type=Point table="public"."ne_10m_populated_places_wgs84" (wkb_geometry) sql="#,
    );
    assert_eq!(
        info.dbconn,
        "postgresql://pi:xxx@localhost:5432/natural_earth_vectors"
    );
    assert_eq!(
        info.table_name,
        r#""public"."ne_10m_populated_places_wgs84""#
    );
    let info = PgLayerInfo::from_qgs_ds("dbname=\'natural_earth_vectors\' port=5432 sslmode=disable key=\'tid\' estimatedmetadata=true srid=3857 type=Polygon table=\"public\".\"admin_0_countries\" (wkb_geometry) sql=");
    assert_eq!(
        info.dbconn,
        format!(
            "postgresql://{}@%2Frun%2Fpostgresql:5432/natural_earth_vectors",
            get_user_name()
        )
    );
    assert_eq!(info.table_name, r#""public"."admin_0_countries""#);
    let info = PgLayerInfo::from_qgs_ds(
        r#"dbname='natural_earth_vectors' port=5432 sslmode=disable key='fid' estimatedmetadata=true srid=4326 type=Point table="public"."ne_10m_populated_places_wgs84" (wkb_geometry) sql="scalerank" &lt; 9"#,
    );
    assert_eq!(
        info.table_name,
        r#""public"."ne_10m_populated_places_wgs84""#
    );
}

#[test]
fn test_gdal_ds() {
    let info = GdalLayerInfo::from_qgs_ds(
        "../examples/natural_earth.qgs",
        "../data/natural_earth.gpkg|layerid=2",
    );
    println!("{:?}", info);
    assert!(info.path.contains("natural_earth.gpkg"));
    assert_eq!(info.layer_name, "ne_110m_admin_0_countries");
    let info = GdalLayerInfo::from_qgs_ds(
        "../examples/natural_earth.qgs",
        "../data/natural_earth.gpkg|layername=ne_10m_rivers_lake_centerlines",
    );
    assert_eq!(info.layer_name, "ne_10m_rivers_lake_centerlines");
    let info = GdalLayerInfo::from_qgs_ds(
        "../examples/natural_earth.qgs",
        "../data/natural_earth.gpkg|layerinfo=missing",
    );
    assert_eq!(
        info.layer_name,
        "<../data/natural_earth.gpkg|layerinfo=missing>"
    );
}

#[test]
fn test_read_qgs() {
    let (dss, ts) = read_qgs("../examples/natural_earth.qgs");
    println!("{}", dss.gen_runtime_config());
    println!("{}", ts.gen_runtime_config());
    assert_eq!(ts.layers.len(), 4);

    assert_eq!(ts.layers[0].name, "admin_0_countries");
    let ref ds = dss.datasources[&ts.layers[0].name];
    let dsconfig = format!(
        r#"
[[datasource]]
dbconn = "postgresql://{}@%2Frun%2Fpostgresql:5432/natural_earth_vectors"
"#,
        get_user_name()
    );
    assert_eq!(ds.gen_runtime_config(), dsconfig);
    let layerconfig = r#"[[tileset.layer]]
name = "admin_0_countries"
datasource = "admin_0_countries"
table_name = "public.admin_0_countries"
geometry_field = "wkb_geometry"
geometry_type = "POLYGON"
srid = 3857
#buffer_size = 10
#make_valid = true
simplify = true
#query_limit = 1000
#[[tileset.layer.query]]
"#;
    assert_eq!(ts.layers[0].gen_runtime_config(), layerconfig);

    let ref ds = dss.datasources[&ts.layers[1].name];
    assert_eq!(ts.layers[1].name, "natural_earth ne_110m_admin_0_countries");
    assert!(ds.gen_runtime_config().contains("natural_earth.gpkg"));
    let layerconfig = r#"[[tileset.layer]]
name = "natural_earth ne_110m_admin_0_countries"
datasource = "natural_earth ne_110m_admin_0_countries"
table_name = "ne_110m_admin_0_countries"
#geometry_field = "wkb_geometry"
#geometry_type = "POINT"
#srid = 3857
#buffer_size = 10
#make_valid = true
simplify = false
#query_limit = 1000
#[[tileset.layer.query]]
"#;
    assert_eq!(ts.layers[1].gen_runtime_config(), layerconfig);
}
