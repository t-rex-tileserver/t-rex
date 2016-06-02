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


pub enum Tilecache {
    Nocache(Nocache),
    Filecache(Filecache),
}

impl Cache for Tilecache {
    fn lookup<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>
    {
        match self {
            &Tilecache::Nocache(ref cache)   => cache.lookup(topic, xtile, ytile, zoom, read),
            &Tilecache::Filecache(ref cache) => cache.lookup(topic, xtile, ytile, zoom, read),
        }
    }
    fn store<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>
    {
        match self {
            &Tilecache::Nocache(ref cache)   => cache.store(topic, xtile, ytile, zoom, write),
            &Tilecache::Filecache(ref cache) => cache.store(topic, xtile, ytile, zoom, write),
        }
    }
}
