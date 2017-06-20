//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod cache;
pub mod filecache;

#[cfg(test)]
mod filecache_test;

pub use self::cache::Cache;
pub use self::cache::Nocache;
pub use self::filecache::Filecache;
use std::io::Read;
use std::io;
use core::ApplicationCfg;
use core::Config;
use toml;


pub enum Tilecache {
    Nocache(Nocache),
    Filecache(Filecache),
}

impl Cache for Tilecache {
    fn info(&self) -> String {
        match self {
            &Tilecache::Nocache(ref cache) => cache.info(),
            &Tilecache::Filecache(ref cache) => cache.info(),
        }
    }
    fn baseurl(&self) -> String {
        match self {
            &Tilecache::Nocache(ref cache) => cache.baseurl(),
            &Tilecache::Filecache(ref cache) => cache.baseurl(),
        }
    }
    fn read<F>(&self, path: &str, read: F) -> bool
        where F: FnMut(&mut Read)
    {
        match self {
            &Tilecache::Nocache(ref cache) => cache.read(path, read),
            &Tilecache::Filecache(ref cache) => cache.read(path, read),
        }
    }
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        match self {
            &Tilecache::Nocache(ref cache) => cache.write(path, obj),
            &Tilecache::Filecache(ref cache) => cache.write(path, obj),
        }
    }
    fn exists(&self, path: &str) -> bool {
        match self {
            &Tilecache::Nocache(ref cache) => cache.exists(path),
            &Tilecache::Filecache(ref cache) => cache.exists(path),
        }
    }
}

impl Config<Tilecache> for Tilecache {
    fn from_cfg(config: &ApplicationCfg) -> Result<Self, String> {
        config.cache.as_ref()
            .map(|cache| {
                     let fc = Filecache {
                         basepath: cache.file.base.clone(),
                         baseurl: cache.file.baseurl.clone(),
                     };
                     Tilecache::Filecache(fc)
                 })
            .or(Some(Tilecache::Nocache(Nocache)))
            .ok_or("".to_string())
    }
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        if let Some(cfg) = config.get("cache").and_then(|c| c.get("file")) {
            cfg.clone()
                .try_into::<Filecache>()
                .and_then(|cache| Ok(Tilecache::Filecache(cache)))
                .map_err(|e| format!("Error reading configuration - {}", e))
        } else {
            Ok(Tilecache::Nocache(Nocache))
        }
    }
    fn gen_config() -> String {
        let toml = r#"
#[cache.file]
#base = "/tmp/mvtcache"
#baseurl = "http://example.com/tiles"
"#;
        toml.to_string()
    }
}
