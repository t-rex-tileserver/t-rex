//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use serde_json;
use std;
use std::collections::BTreeMap;
use toml::map::Map;
use toml::Value::{self, Array, Boolean, Datetime, Float, Integer, String, Table};

/// Convert Mapbox GL Styles from [TOML format](https://pka.github.io/mapbox-gl-style-spec/) to JSON
pub fn toml_style_to_gljson(toml: &toml::Value) -> std::string::String {
    let converter = TomlConverter::new();
    let json = converter.convert_value(toml);
    serde_json::to_string_pretty(&json).unwrap()
}

struct TomlConverter;
impl TomlConverter {
    pub fn new() -> TomlConverter {
        TomlConverter
    }

    pub fn convert_value(&self, toml: &toml::Value) -> serde_json::Value {
        let json = match *toml {
            Table(ref value) => serde_json::to_value(self.convert_table(value)),

            Array(ref array) => {
                let mut vec = Vec::new();
                for value in array.iter() {
                    vec.push(self.convert_value(value));
                }
                serde_json::to_value(vec)
            }

            String(ref value) => serde_json::to_value(value),
            Integer(ref value) => serde_json::to_value(value),
            Float(ref value) => serde_json::to_value(value),
            Boolean(ref value) => serde_json::to_value(value),
            Datetime(ref value) => serde_json::to_value(value),
        };
        json.unwrap()
    }

    pub fn convert_table(&self, table: &Map<std::string::String, Value>) -> serde_json::Value {
        let mut json: BTreeMap<std::string::String, serde_json::Value> = BTreeMap::new();
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
                                        self.convert_value(&stops_tbl["out"]),
                                    ]);
                                } else {
                                    stops.push(vec![self.convert_value(&stops_tbl["in"])]);
                                }
                            }
                        }
                    }
                }
                json.insert(key.to_string(), serde_json::to_value(stops).unwrap());
            } else {
                json.insert(key.to_string(), self.convert_value(value));
            }
        }
        serde_json::to_value(json).unwrap()
    }
}
