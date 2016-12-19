//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml;
use rustc_serialize::json::{self, Json};


/// Convert Mapbox GL Styles from [TOML format](https://pka.github.io/mapbox-gl-style-spec/) to JSON
pub fn toml_style_to_gljson(toml: &toml::Value) -> String {
    let configjson = json::encode(toml).unwrap().replace("}{", "},{").replace("][", "],[");
    Json::from_str(&configjson).unwrap().pretty().to_string()
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
      {
        "in": 0,
        "out": "blue"
      },
      {
        "in": 100,
        "out": "red"
      }
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
        stops = [{ in = 15 }, { in = 16 }]"##;

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
          {
            "in": 15,
            "out": 0
          },
          {
            "in": 16,
            "out": 1
          }
        ]
      },
      "fill-outline-color": "#dfdbd7",
      "fill-translate": {
        "base": 1,
        "stops": [
          {
            "in": 15
          },
          {
            "in": 16
          }
        ]
      }
    }
  }
}"##;
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

    let style = config.lookup("tileset.0.layer.1.style").unwrap();

    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected= r##"{
  "interactive": true,
  "paint": {
    "fill-color": {
      "base": 1,
      "stops": [
        {
          "in": 15.5,
          "out": "#f2eae2"
        },
        {
          "in": 16,
          "out": "#dfdbd7"
        }
      ]
    }
  },
  "type": "fill"
}"##;
    assert_eq!(configjson, expected);
}
