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
