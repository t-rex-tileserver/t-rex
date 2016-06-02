//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod cache;
pub mod filecache;

pub use self::cache::Cache;
pub use self::cache::Nocache;
pub use self::filecache::Filecache;
use std::io::{Read,Write};
use std::io;
use core::Config;
use toml;


pub enum Tilecache {
    Nocache(Nocache),
    Filecache(Filecache),
}

impl Cache for Tilecache {
    fn lookup<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, mut read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>
    {
        match self {
            &Tilecache::Nocache(ref cache)   => cache.lookup(tileset, xtile, ytile, zoom, read),
            &Tilecache::Filecache(ref cache) => cache.lookup(tileset, xtile, ytile, zoom, read),
        }
    }
    fn store<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, mut write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>
    {
        match self {
            &Tilecache::Nocache(ref cache)   => cache.store(tileset, xtile, ytile, zoom, write),
            &Tilecache::Filecache(ref cache) => cache.store(tileset, xtile, ytile, zoom, write),
        }
    }
}

impl Config<Tilecache> for Tilecache {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        config.lookup("cache.file.base")
            .and_then(|val| val.as_str().or(None))
            .and_then(|basedir| Some(Tilecache::Filecache(Filecache {basepath: basedir.to_string() })))
            .or( Some(Tilecache::Nocache(Nocache)) )
            .ok_or("config error".to_string())
    }
    fn gen_config() -> String {
        let toml = r#"
#[cache.file]
#base = "/tmp/mvtcache"
"#;
        toml.to_string()
    }
}
