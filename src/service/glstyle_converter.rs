//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::collections::BTreeMap;
use rustc_serialize::json::{ToJson, Json};
use toml;
use std;

use toml::Value::{self, String, Integer, Float, Boolean, Datetime, Array, Table};


/// Convert Mapbox GL Styles from [TOML format](https://pka.github.io/mapbox-gl-style-spec/) to JSON
pub fn toml_style_to_gljson(toml: &toml::Value) -> std::string::String {
    let converter = TomlConverter::new();
    let json = converter.convert_value(toml);
    json.pretty().to_string()
}


struct TomlConverter;
impl TomlConverter {
    pub fn new() -> TomlConverter {
        TomlConverter
    }

    pub fn convert_value(&self, toml: &toml::Value) -> Json {
        match *toml {
            Table(ref value) => self.convert_table(value),

            Array(ref array) => {
                let mut vec = Vec::new();
                for value in array.iter() {
                    vec.push(self.convert_value(value));
                }
                vec.to_json()
            }

            String(ref value) => value.to_json(),
            Integer(ref value) => value.to_json(),
            Float(ref value) => value.to_json(),
            Boolean(ref value) => value.to_json(),
            Datetime(ref value) => value.to_json(),
        }
    }

    pub fn convert_table(&self, table: &BTreeMap<std::string::String, Value>) -> Json {
        let mut json: BTreeMap<std::string::String, Json> = BTreeMap::new();
        for (key, value) in table.iter() {
            if key == "stops" {
                let mut stops = Vec::new();
                if let Array(ref stops_arr) = *value {
                    for stop in stops_arr.iter() {
                        if let Table(ref stops_tbl) = *stop {
                            if stops_tbl.contains_key("in") {
                                if stops_tbl.contains_key("out") {
                                    stops.push(vec![
                                        self.convert_value(&stops_tbl["in"]),
                                        self.convert_value(&stops_tbl["out"])]);
                                } else {
                                    stops.push(vec![
                                        self.convert_value(&stops_tbl["in"])]);
                                }
                            }
                        }
                    }

                }
                json.insert(key.to_string(), stops.to_json());
            } else {
                json.insert(key.to_string(), self.convert_value(value));
            }
        }
        json.to_json()
    }
}


#[test]
pub fn color_stops() {
    use toml::{Value, Parser};

    let style = r##"
        [circle-color]
        property = "temperature"
        stops = [{ in = 0, out = "blue" }, { in = 100, out = "red" }]"##;

    let mut parser = Parser::new(style);
    let toml = Value::Table(parser.parse().unwrap());

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected= r##"{
  "circle-color": {
    "property": "temperature",
    "stops": [
      [
        0,
        "blue"
      ],
      [
        100,
        "red"
      ]
    ]
  }
}"##;
    assert_eq!(configjson, expected);

    let style = r##"
        [layers.paint]
        fill-color = "#f2eae2"
        fill-outline-color = "#dfdbd7"

        [layers.paint.fill-opacity]
        base = 1
        stops = [{ in = 15, out = 0 }, { in = 16, out = 1 }]

        [layers.paint.fill-translate]
        base = 1
        stops = [{ in = 15, out = [0,0] }, { in = 16, out = [-2,-2] }]"##;

    let mut parser = Parser::new(style);
    let toml = Value::Table(parser.parse().unwrap());

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected= r##"{
  "layers": {
    "paint": {
      "fill-color": "#f2eae2",
      "fill-opacity": {
        "base": 1,
        "stops": [
          [
            15,
            0
          ],
          [
            16,
            1
          ]
        ]
      },
      "fill-outline-color": "#dfdbd7",
      "fill-translate": {
        "base": 1,
        "stops": [
          [
            15,
            [
              0,
              0
            ]
          ],
          [
            16,
            [
              -2,
              -2
            ]
          ]
        ]
      }
    }
  }
}"##;
    assert_eq!(configjson, expected);
}

#[test]
pub fn filters() {
    use toml::{Value, Parser};

    let style = r##"
        [[layers]]
        id = "tunnel_path_pedestrian"
        type = "line"
        source = "mapbox"
        source-layer = "road"
        filter = [["all"], ["==", "$type", "LineString"], [["all"], ["==", "structure", "tunnel"], ["in", "class", "path", "pedestrian"]]]
        interactive = true"##;

    let mut parser = Parser::new(style);
    let toml = Value::Table(parser.parse().unwrap());

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected= r##"{
  "layers": [
    {
      "filter": [
        [
          "all"
        ],
        [
          "==",
          "$type",
          "LineString"
        ],
        [
          [
            "all"
          ],
          [
            "==",
            "structure",
            "tunnel"
          ],
          [
            "in",
            "class",
            "path",
            "pedestrian"
          ]
        ]
      ],
      "id": "tunnel_path_pedestrian",
      "interactive": true,
      "source": "mapbox",
      "source-layer": "road",
      "type": "line"
    }
  ]
}"##;
    /* TODO: Should be
    "id": "tunnel_path_pedestrian",
    "type": "line",
    "source": "mapbox",
    "source-layer": "road",
    "filter": [
        "all",
        [
            "==",
            "$type",
            "LineString"
        ],
        [
            "all",
            [
                "==",
                "structure",
                "tunnel"
            ],
            [
                "in",
                "class",
                "path",
                "pedestrian"
            ]
        ]
    ],*/
    assert_eq!(configjson, expected);
}

#[test]
pub fn layer_style_from_cfg() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();

    let style = config.lookup("tileset.0.style").unwrap();

    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected= r##"{
  "paint": {
    "background-color": "#f8f4f0"
  },
  "type": "background"
}"##;
    assert_eq!(configjson, expected);

    let style = config.lookup("tileset.0.layer.2.style").unwrap();

    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected= r##"{
  "interactive": true,
  "paint": {
    "fill-color": "#d8e8c8",
    "fill-opacity": 0.5
  },
  "type": "fill"
}"##;
    assert_eq!(configjson, expected);
}
