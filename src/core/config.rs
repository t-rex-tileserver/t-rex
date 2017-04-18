//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml::Value;
use std::io::prelude::*;
use std::fs::File;


pub trait Config<T> {
    /// Read configuration
    fn from_config(config: &Value) -> Result<T, String>;
    /// Generate configuration template
    fn gen_config() -> String;
    /// Generate configuration template with runtime information
    fn gen_runtime_config(&self) -> String {
        Self::gen_config()
    }
}

/// Load and parse the config file into Toml table structure.
pub fn read_config(path: &str) -> Result<Value, String> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_)  => {
            return Err("Could not find config file!".to_string());
        }
    };
    let mut config_toml = String::new();
    if let Err(err) = file.read_to_string(&mut config_toml) {
        return Err(format!("Error while reading config: [{}]", err));
    };

    parse_config(config_toml, path)
}

/// Parse the configuration into Toml table structure.
pub fn parse_config(config_toml: String, path: &str) -> Result<Value, String> {
    let toml = config_toml.parse::<Value>();
    /* FIXME:
    if toml.is_none() {
        let mut errors = Vec::new();
        for err in &parser.errors {
            let (loline, locol) = parser.to_linecol(err.lo);
            let (hiline, hicol) = parser.to_linecol(err.hi);
            errors.push(format!("{}:{}:{}-{}:{} error: {}",
                     path, loline, locol, hiline, hicol, err.desc));
        }
        return Err(errors.join("\n"));
    }*/

    Ok(toml.unwrap())
 }
