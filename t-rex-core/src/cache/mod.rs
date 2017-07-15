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
use core::Config;
use core::ApplicationCfg;


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

impl<'a> Config<'a, Tilecache, ApplicationCfg> for Tilecache {
    fn from_config(config: &ApplicationCfg) -> Result<Self, String> {
        config
            .cache
            .as_ref()
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
    fn gen_config() -> String {
        let toml = r#"
#[cache.file]
#base = "/tmp/mvtcache"
#baseurl = "http://example.com/tiles"
"#;
        toml.to_string()
    }
}
