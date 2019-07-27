//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::service::glstyle_converter::toml_style_to_gljson;

#[test]
pub fn color_stops() {
    use toml::Value;

    let style = r##"
        [circle-color]
        property = "temperature"
        stops = [{ in = 0, out = "blue" }, { in = 100, out = "red" }]"##;

    let toml = style.parse::<Value>().unwrap();

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected = r##"{
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

    let toml = style.parse::<Value>().unwrap();

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected = r##"{
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
    use toml::Value;

    let style = r##"
        [[layers]]
        id = "tunnel_path_pedestrian"
        type = "line"
        source = "mapbox"
        source-layer = "road"
        filter = [["all"], ["==", "$type", "LineString"], [["all"], ["==", "structure", "tunnel"], ["in", "class", "path", "pedestrian"]]]
        interactive = true"##;

    let toml = style.parse::<Value>().unwrap();

    let configjson = toml_style_to_gljson(&toml);
    println!("{}", configjson);
    let expected = r##"{
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
pub fn layer_style_from_config() {
    use crate::core::config::ApplicationCfg;
    use crate::core::read_config;

    let config: ApplicationCfg = read_config("../t-rex-service/src/test/example.toml").unwrap();

    let ref style = config.tilesets[0].style.clone().unwrap();

    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected = r##"{
  "paint": {
    "background-color": "#f8f4f0"
  },
  "type": "background"
}"##;
    assert_eq!(configjson, expected);

    let ref style = config.tilesets[0].layers[2].style.clone().unwrap();

    let configjson = toml_style_to_gljson(style);
    println!("{}", configjson);
    let expected = r##"{
  "paint": {
    "fill-color": "#d8e8c8",
    "fill-opacity": 0.5
  },
  "type": "fill"
}"##;
    assert_eq!(configjson, expected);
}
