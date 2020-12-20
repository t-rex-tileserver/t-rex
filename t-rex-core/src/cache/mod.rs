//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

pub mod cache;
pub mod filecache;
pub mod s3cache;


#[cfg(test)]
mod filecache_test;
mod s3cache_test;

pub use self::cache::Cache;
pub use self::cache::Nocache;
pub use self::filecache::Filecache;
pub use self::s3cache::S3Cache;
use crate::core::ApplicationCfg;
use crate::core::Config;
use std::io;
use std::io::Read;

#[derive(Clone)]
pub enum Tilecache {
    Nocache(Nocache),
    Filecache(Filecache),
    S3Cache(S3Cache)
}

impl Cache for Tilecache {
    fn info(&self) -> String {
        match self {
            &Tilecache::Nocache(ref cache) => cache.info(),
            &Tilecache::Filecache(ref cache) => cache.info(),
            &Tilecache::S3Cache(ref cache) => cache.info(),
        }
    }
    fn baseurl(&self) -> String {
        match self {
            &Tilecache::Nocache(ref cache) => cache.baseurl(),
            &Tilecache::Filecache(ref cache) => cache.baseurl(),
            &Tilecache::S3Cache(ref cache) => cache.baseurl(),
        }
    }
    fn read<F>(&self, path: &str, read: F) -> bool
    where
        F: FnMut(&mut dyn Read),
    {
        match self {
            &Tilecache::Nocache(ref cache) => cache.read(path, read),
            &Tilecache::Filecache(ref cache) => cache.read(path, read),
            &Tilecache::S3Cache(ref cache) => cache.read(path, read),

        }
    }
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        match self {
            &Tilecache::Nocache(ref cache) => cache.write(path, obj),
            &Tilecache::Filecache(ref cache) => cache.write(path, obj),
            &Tilecache::S3Cache(ref cache) => cache.write(path, obj),
        }
    }
    fn exists(&self, path: &str) -> bool {
        match self {
            &Tilecache::Nocache(ref cache) => cache.exists(path),
            &Tilecache::Filecache(ref cache) => cache.exists(path),
            &Tilecache::S3Cache(ref cache) => cache.exists(path),
        }
    }
}

impl<'a> Config<'a, ApplicationCfg> for Tilecache {
    fn from_config(config: &ApplicationCfg) -> Result<Self, String> {
        config
            .cache
            .as_ref()
            .map(|cache| {
                let fc = Filecache {
                    basepath: cache.file.as_ref().unwrap().base.clone(),
                    baseurl: cache.file.as_ref().unwrap().baseurl.clone(),
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
