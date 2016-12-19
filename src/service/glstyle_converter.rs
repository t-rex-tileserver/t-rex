//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml;
use rustc_serialize::json::{self, Json};

pub fn toml_style_to_gljson(toml: &toml::Value) -> String {
    let configjson = json::encode(toml).unwrap().replace("}{", "},{").replace("][", "],[");
    Json::from_str(&configjson).unwrap().pretty().to_string()
}

#[test]
pub fn test_stylejson() {
    use core::read_config;

    let config = read_config("src/test/example.cfg").unwrap();
    let style = config.lookup("tileset.0.layer.1.style").unwrap();

    // Mapbox GL style experiments
    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected= r##"[
  {
    "fill-color": {
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
    },
    "interactive": true,
    "type": "fill"
  },
  {
    "circle-color": [
      {
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
    ],
    "fill-color": "#f2eae2",
    "fill-outline-color": "#dfdbd7",
    "fill-translate": {
      "stops": [
        {
          "in": 15,
          "out": [
            11
          ]
        },
        {
          "in": 16,
          "out": [
            -20
          ]
        }
      ]
    },
    "fillopacity": {
      "base": 1,
      "stops": [
        [
          150
        ],
        [
          161
        ]
      ]
    },
    "interactive": true,
    "type": "fill"
  }
]"##;
    assert_eq!(configjson, expected);
}
