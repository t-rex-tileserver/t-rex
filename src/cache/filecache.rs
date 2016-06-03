//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use cache::cache::Cache;
use std::fs::{self,File};
use std::io::{self,Read,Write};
use std::path::Path;


pub struct Filecache {
    pub basepath: String,
}

impl Filecache {
    fn dir(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16) -> String {
        format!("{}/{}/{}/{}", self.basepath, tileset, zoom, xtile)
    }
    fn path(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16) -> String {
        format!("{}/{}.pbf", self.dir(tileset, xtile, ytile, zoom), ytile)
    }
}

impl Cache for Filecache {
    fn lookup<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, mut read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>
    {
        let path = self.path(tileset, xtile, ytile, zoom);
        debug!("Filecache.lookup {}", path);
        match File::open(&path) {
            Ok(mut f) => read(&mut f),
            Err(e) => Err(e)
        }
    }
    fn store<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, mut write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>
    {
        let path = self.path(tileset, xtile, ytile, zoom);
        debug!("Filecache.store {}", path);
        let dir = self.dir(tileset, xtile, ytile, zoom);
        try!(fs::create_dir_all(Path::new(&dir as &str)));
        let mut f = try!(File::create(path));
        write(&mut f)
    }
}


#[test]
fn test_file() {
    use std::env;

    let mut dir = env::temp_dir();
    dir.push("t_rex_test");
    let basepath = format!("{}", &dir.display());
    fs::remove_dir_all(&basepath);

    let cache = Filecache { basepath: basepath };
    assert_eq!(cache.dir("tileset", 1, 2, 0), format!("{}/{}", cache.basepath, "tileset/0/1"));
    let pbf = format!("{}/{}", cache.basepath, "tileset/0/1/2.pbf");
    assert_eq!(cache.path("tileset", 1, 2, 0), pbf);

    // Cache miss
    assert!(cache.lookup("tileset", 1, 2, 0, |_| Ok(())).is_err());

    // Write into cache
    let res = cache.store("tileset", 1, 2, 0, |f| {
        f.write_all("0123456789".as_bytes())
    });
    assert_eq!(res.ok(), Some(()));
    assert!(Path::new(&pbf).exists());

    // Cache hit
    assert!(cache.lookup("tileset", 1, 2, 0, |_| Ok(())).is_ok());

    // Read from cache
    let mut s = String::new();
    cache.lookup("tileset", 1, 2, 0, |f| {
        f.read_to_string(&mut s).map(|_| ())
    });
    assert_eq!(&s, "0123456789");
}
