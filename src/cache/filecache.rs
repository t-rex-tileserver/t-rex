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

impl Cache for Filecache {
    fn read<F>(&self, path: &str, mut read: F) -> bool
        where F : FnMut(&mut Read)
    {
        let fullpath = format!("{}/{}", self.basepath, path);
        debug!("Filecache.read {}", fullpath);
        match File::open(&fullpath) {
            Ok(mut f) => { read(&mut f); true },
            Err(_e) => false
        }
    }
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error>
    {
        let fullpath = format!("{}/{}", self.basepath, path);
        debug!("Filecache.write {}", fullpath);
        let p = Path::new(&fullpath);
        try!(fs::create_dir_all(p.parent().unwrap()));
        let mut f = try!(File::create(&fullpath));
        f.write_all(obj)
    }

    fn exists(&self, path: &str) -> bool {
        let fullpath = format!("{}/{}", self.basepath, path);
        Path::new(&fullpath).exists()
    }
}

#[test]
fn test_dircache() {
    use std::env;

    let mut dir = env::temp_dir();
    dir.push("t_rex_test");
    let basepath = format!("{}", &dir.display());
    let _ = fs::remove_dir_all(&basepath);

    let cache = Filecache { basepath: basepath };
    let path = "tileset/0/1/2.pbf";
    let fullpath = format!("{}/{}", cache.basepath, path);
    let obj = "0123456789";

    // Cache miss
    assert_eq!(cache.read(path, |_| {}), false);

    // Write into cache
    let _ = cache.write(path, obj.as_bytes());
    assert!(Path::new(&fullpath).exists());

    // Cache hit
    assert_eq!(cache.read(path, |_| {}), true);

    // Read from cache
    let mut s = String::new();
    cache.read(path, |f| {
        let _ = f.read_to_string(&mut s);
    });
    assert_eq!(&s, "0123456789");
}
