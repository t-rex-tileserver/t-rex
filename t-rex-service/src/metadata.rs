//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::mvt_service::MvtService;
use serde_json;
use std::cmp;
use t_rex_core::datasource::DatasourceType;

type JsonResult = Result<serde_json::Value, serde_json::error::Error>;

impl MvtService {
    /// Service metadata for backend web application
    pub fn get_mvt_metadata(&self) -> JsonResult {
        #[derive(Serialize)]
        struct MvtInfo {
            tilesets: Vec<TilesetInfo>,
        }
        #[derive(Serialize)]
        struct TilesetInfo {
            name: String,
            tilejson: String,
            tileurl: String,
            bounds: [f64; 4],
            layers: Vec<LayerInfo>,
            supported: bool,
        }
        #[derive(Serialize)]
        struct LayerInfo {
            name: String,
            geometry_type: Option<String>,
        }

        let mut tileset_infos: Vec<TilesetInfo> = self
            .tilesets
            .iter()
            .map(|set| {
                let layerinfos = set
                    .layers
                    .iter()
                    .map(|l| LayerInfo {
                        name: l.name.clone(),
                        geometry_type: l.geometry_type.clone(),
                    })
                    .collect();
                let supported = set.layers.iter().any(|l| {
                    let geom_type = l.geometry_type.clone().unwrap_or("UNKNOWN".to_string());
                    ["POINT", "LINESTRING", "POLYGON"].contains(&(&geom_type as &str))
                });
                let ext = set.get_extent();
                TilesetInfo {
                    name: set.name.clone(),
                    tilejson: format!("{}.json", set.name),
                    tileurl: format!("/{}/{{z}}/{{x}}/{{y}}.pbf", set.name),
                    bounds: [ext.minx, ext.miny, ext.maxx, ext.maxy],
                    layers: layerinfos,
                    supported: supported,
                }
            })
            .collect();
        tileset_infos.sort_by_key(|ti| ti.name.clone());
        let mvt_info = MvtInfo {
            tilesets: tileset_infos,
        };
        serde_json::to_value(mvt_info)
    }
    fn get_tilejson_metadata(&self, tileset: &str) -> JsonResult {
        let ts = self
            .get_tileset(tileset)
            .expect(&format!("Tileset '{}' not found", tileset));
        let ext = ts.get_extent();
        let center = ts.get_center();
        let zoom = ts.get_start_zoom();
        Ok(json!({
            "id": tileset,
            "name": tileset,
            "description": tileset,
            "attribution": ts.attribution(),
            "format": "pbf",
            "version": "2.0.0",
            "scheme": "xyz",
            "bounds": [ext.minx,
                       ext.miny,
                       ext.maxx,
                       ext.maxy],
            // Minimum zoom level for which tiles are available.
            // Optional. Default: 0. >= 0, <= 30.
            "minzoom": ts.minzoom(),
            // Maximum zoom level for which tiles are available.
            // Data from tiles at the maxzoom are used when displaying the map at higher zoom levels.
            // Optional. Default: 30. >= 0, <= 30. (Mapbox Style default: 22)
            "maxzoom": ts.maxzoom(),
            "center": [center.0, center.1, zoom],
            "basename": tileset
        }))
    }
    fn get_tilejson_layers(&self, tileset: &str) -> JsonResult {
        let ts = self
            .get_tileset(tileset)
            .expect(&format!("Tileset '{}' not found", tileset));
        let layers = self.get_tileset_layers(tileset);
        let layers_metadata: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let meta = layer.metadata();
                let query = layer.query(layer.maxzoom(22));
                let mut meta_json = json!({
                    "id": meta.get("id").unwrap(),
                    "name": meta.get("name").unwrap(),
                    "description": meta.get("description").unwrap(),
                    "srs": meta.get("srs").unwrap(),
                    "properties": {
                        "minzoom": cmp::max(ts.minzoom(), layer.minzoom()),
                        "maxzoom": cmp::min(ts.maxzoom(), layer.maxzoom(22)),
                        "buffer-size": layer.buffer_size.unwrap_or(0)
                    },
                    "fields": {}
                });
                //insert fields
                let fields = self.ds(&layer).unwrap().detect_data_columns(&layer, query);
                for (ref field, _) in fields {
                    meta_json["fields"]
                        .as_object_mut()
                        .unwrap()
                        .insert(field.clone(), json!(""));
                }
                meta_json
            })
            .collect();
        Ok(json!(layers_metadata))
    }
    // MVT layers in TileJSON manifest
    // https://github.com/mapbox/tilejson-spec/tree/3.0-vector_layers/3.0#315-vector_layers
    fn get_tilejson_vector_layers(&self, tileset: &str) -> JsonResult {
        let ts = self
            .get_tileset(tileset)
            .expect(&format!("Tileset '{}' not found", tileset));
        let layers = self.get_tileset_layers(tileset);
        let vector_layers: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let meta = layer.metadata();
                let query = layer.query(layer.maxzoom(22));
                let mut layer_json = json!({
                    "id": meta.get("id").unwrap(),
                    "description": meta.get("description").unwrap(), // Optional
                    // lowest zoom level whose tiles this layer appears in.
                    // must be greater than or equal to the tileset's minzoom
                    "minzoom": cmp::max(ts.minzoom(), layer.minzoom()),
                    // highest zoom level whose tiles this layer appears in.
                    // must  be less than or equal to the tileset's maxzoom
                    "maxzoom": cmp::min(ts.maxzoom(), layer.maxzoom(22)),
                    "fields": {}
                });
                if let Some(srid) = layer.srid {
                    layer_json["projection"] = json!(format!("EPSG:{}", srid));
                }
                //insert fields
                let fields = self.ds(&layer).unwrap().detect_data_columns(&layer, query);
                for (ref field, _) in fields {
                    layer_json["fields"]
                        .as_object_mut()
                        .unwrap()
                        .insert(field.clone(), json!(""));
                }
                layer_json
            })
            .collect();
        Ok(json!(vector_layers))
    }
    /// TileJSON metadata (https://github.com/mapbox/tilejson-spec)
    pub fn get_tilejson(&self, baseurl: &str, tileset: &str) -> JsonResult {
        let mut metadata = self.get_tilejson_metadata(tileset)?;
        let vector_layers = self.get_tilejson_vector_layers(tileset)?;
        let url = json!([format!("{}/{}/{{z}}/{{x}}/{{y}}.pbf", baseurl, tileset)]);
        let obj = metadata.as_object_mut().unwrap();
        obj.insert("tiles".to_string(), url);
        obj.insert("vector_layers".to_string(), vector_layers);
        Ok(json!(obj))
    }
    /// MapboxGL Style JSON (https://www.mapbox.com/mapbox-gl-style-spec/)
    pub fn get_stylejson(&self, baseurl: &str, tileset: &str) -> JsonResult {
        let mut stylejson = json!({
            "version": 8,
            "name": "t-rex",
            "metadata": {
                // prevent compositing in mapbox studio
                "mapbox:autocomposite": false,
                "maputnik:renderer": "mbgljs"
            },
            "glyphs": format!("{}/fonts/{{fontstack}}/{{range}}.pbf", baseurl),
            "sources": {
                tileset: {
                    "url": format!("{}/{}.json", baseurl, tileset),
                    "type": "vector"
                }
            }
        });
        let background_layer = json!({
          "id": "background_",
          "type": "background",
          "paint": {
            "background-color": "rgba(255, 255, 255, 1)"
          }
        }); // TODO: add style.background-color element
        let layers = self.get_tileset_layers(tileset);
        let mut layer_styles: Vec<serde_json::Value> = layers
            .iter()
            .map(|layer| {
                let mut layerjson = if let Some(ref style) = layer.style {
                    serde_json::from_str(&style).unwrap()
                } else {
                    json!({})
                };
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("id".to_string(), json!(layer.name));
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("source".to_string(), json!(tileset));
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .insert("source-layer".to_string(), json!(layer.name));
                // Note: source-layer referencing other layers not supported

                // minzoom:
                // The minimum zoom level for the layer. At zoom levels less than the minzoom, the layer will be hidden.
                // Optional number between 0 and 24 inclusive.
                // maxzoom:
                // The maximum zoom level for the layer. At zoom levels equal to or greater than the maxzoom, the layer will be hidden.
                // Optional number between 0 and 24 inclusive.
                // Note: We could use source data min-/maxzoom as default to prevent overzooming
                // or we could add style.minzoom, style.maxzoom elements

                // Default paint type
                let default_type = if let Some(ref geomtype) = layer.geometry_type {
                    match &geomtype as &str {
                        "POINT" => "circle",
                        _ => "line",
                    }
                } else {
                    "line"
                }
                .to_string();
                layerjson
                    .as_object_mut()
                    .unwrap()
                    .entry("type".to_string())
                    .or_insert(json!(default_type));

                layerjson
            })
            .collect();
        layer_styles.insert(0, background_layer);
        // Insert layers in stylejson
        let obj = stylejson.as_object_mut().unwrap();
        obj.insert("layers".to_string(), json!(layer_styles));
        Ok(json!(obj))
    }

    /// MBTiles metadata.json (https://github.com/mapbox/mbtiles-spec/blob/master/1.3/spec.md)
    pub fn get_mbtiles_metadata(&self, tileset: &str) -> JsonResult {
        let mut metadata = self.get_tilejson_metadata(tileset)?;
        metadata["bounds"] = json!(metadata["bounds"].to_string());
        metadata["center"] = json!(metadata["center"].to_string());
        let layers = self.get_tilejson_layers(tileset)?;
        let vector_layers = self.get_tilejson_vector_layers(tileset)?;
        let metadata_vector_layers = json!({
            "Layer": layers,
            "vector_layers": vector_layers
        });
        let obj = metadata.as_object_mut().unwrap();
        obj.insert(
            "json".to_string(),
            json!(metadata_vector_layers.to_string()),
        );
        Ok(json!(obj))
    }
}

#[cfg(test)]
use t_rex_core::core::Config;

#[test]
fn test_mvt_metadata() {
    use t_rex_core::core::read_config;

    let config = read_config("src/test/example.toml").unwrap();
    let service = MvtService::from_config(&config).unwrap();

    let metadata = format!("{:#}", service.get_mvt_metadata().unwrap());
    let expected = r#"{
  "tilesets": [
    {
      "bounds": [
        -180.0,
        -90.0,
        180.0,
        90.0
      ],
      "layers": [
        {
          "geometry_type": "POINT",
          "name": "points"
        },
        {
          "geometry_type": "POLYGON",
          "name": "buildings"
        },
        {
          "geometry_type": "POLYGON",
          "name": "admin_0_countries"
        }
      ],
      "name": "osm",
      "supported": true,
      "tilejson": "osm.json",
      "tileurl": "/osm/{z}/{x}/{y}.pbf"
    }
  ]
}"#;
    println!("{}", metadata);
    assert_eq!(metadata, expected);
}

#[test]
#[ignore]
fn test_tilejson() {
    use std::env;
    use t_rex_core::core::read_config;

    env::var("DBCONN").expect("DBCONN undefined");
    let config = read_config("src/test/example.toml").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    service.prepare_feature_queries();
    let metadata = format!(
        "{:#}",
        service.get_tilejson("http://127.0.0.1", "osm").unwrap()
    );
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": [
    -180.0,
    -90.0,
    180.0,
    90.0
  ],
  "center": [
    0.0,
    0.0,
    2
  ],
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "maxzoom": 22,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "tiles": [
    "http://127.0.0.1/osm/{z}/{x}/{y}.pbf"
  ],
  "vector_layers": [
    {
      "description": "",
      "fields": {},
      "id": "points",
      "maxzoom": 22,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "buildings",
      "maxzoom": 22,
      "minzoom": 0
    },
    {
      "description": "",
      "fields": {},
      "id": "admin_0_countries",
      "maxzoom": 22,
      "minzoom": 0,
      "projection": "EPSG:3857"
    }
  ],
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}

#[test]
fn test_stylejson() {
    use t_rex_core::core::read_config;

    let config = read_config("src/test/example.toml").unwrap();
    let service = MvtService::from_config(&config).unwrap();
    let json = format!(
        "{:#}",
        service.get_stylejson("http://127.0.0.1", "osm").unwrap()
    );
    println!("{}", json);
    let expected = r#"
  "name": "t-rex",
  "sources": {
    "osm": {
      "type": "vector",
      "url": "http://127.0.0.1/osm.json"
    }
  },
  "version": 8
"#;
    assert!(json.contains(expected));
    let expected = r#"
  "layers": [
    {
      "id": "background_",
      "paint": {
        "background-color": "rgba(255, 255, 255, 1)"
      },
      "type": "background"
    },
    {
      "id": "points","#;
    assert!(json.contains(expected));

    let expected = r##"
      "paint": {
        "fill-color": "#d8e8c8",
        "fill-opacity": 0.5
      },"##;
    assert!(json.contains(expected));

    let expected = r#"
      "id": "buildings","#;
    assert!(json.contains(expected));
}

#[test]
#[ignore]
fn test_mbtiles_metadata() {
    use std::env;
    use t_rex_core::core::read_config;

    env::var("DBCONN").expect("DBCONN undefined");
    let config = read_config("src/test/example.toml").unwrap();
    let mut service = MvtService::from_config(&config).unwrap();
    service.connect();
    let metadata = format!("{:#}", service.get_mbtiles_metadata("osm").unwrap());
    println!("{}", metadata);
    let expected = r#"{
  "attribution": "",
  "basename": "osm",
  "bounds": "[-180.0,-90.0,180.0,90.0]",
  "center": "[0.0,0.0,2]",
  "description": "osm",
  "format": "pbf",
  "id": "osm",
  "json": "{\"Layer\":[{\"description\":\"\",\"fields\":{},\"id\":\"points\",\"name\":\"points\",\"properties\":{\"buffer-size\":0,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"name\":\"buildings\",\"properties\":{\"buffer-size\":10,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"},{\"description\":\"\",\"fields\":{},\"id\":\"admin_0_countries\",\"name\":\"admin_0_countries\",\"properties\":{\"buffer-size\":1,\"maxzoom\":22,\"minzoom\":0},\"srs\":\"+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 +y_0=0.0 +k=1.0 +units=m +nadgrids=@null +wktext +no_defs +over\"}],\"vector_layers\":[{\"description\":\"\",\"fields\":{},\"id\":\"points\",\"maxzoom\":22,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"buildings\",\"maxzoom\":22,\"minzoom\":0},{\"description\":\"\",\"fields\":{},\"id\":\"admin_0_countries\",\"maxzoom\":22,\"minzoom\":0,\"projection\":\"EPSG:3857\"}]}",
  "maxzoom": 22,
  "minzoom": 0,
  "name": "osm",
  "scheme": "xyz",
  "version": "2.0.0"
}"#;
    assert_eq!(metadata, expected);
}
