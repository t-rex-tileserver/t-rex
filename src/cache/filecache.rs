//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use cache::cache::Cache;
use std::fs::File;
use std::fs;
use std::io::{Read,Write};
use std::io;
use std::path::Path;


pub struct Filecache<'a> {
    pub basepath: &'a str,
}

impl<'a> Filecache<'a> {
    fn dir(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> String {
        format!("{}/{}/{}", self.basepath, zoom, xtile)
    }
    fn path(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> String {
        format!("{}/{}.pbf", self.dir(topic, xtile, ytile, zoom), ytile)
    }
}

impl<'a> Cache for Filecache<'a> {
    fn lookup<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>
    {
        match File::open(&self.path(topic, xtile, ytile, zoom)) {
            Ok(mut f) => read(&mut f),
            Err(e) => Err(e)
        }
    }
    fn store<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>
    {
        let dir = self.dir(topic, xtile, ytile, zoom);
        try!(fs::create_dir_all(Path::new(&dir as &str)));
        let mut f = try!(File::create(self.path(topic, xtile, ytile, zoom)));
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

    let cache = Filecache { basepath: &basepath };
    assert_eq!(cache.dir("topic", 1, 2, 0), format!("{}/{}", cache.basepath, "0/1"));
    let pbf = format!("{}/{}", cache.basepath, "0/1/2.pbf");
    assert_eq!(cache.path("topic", 1, 2, 0), pbf);

    // Cache empty
    assert!(cache.lookup("topic", 1, 2, 0, |_| Ok(())).is_err());

    // Write into cache
    let res = cache.store("topic", 1, 2, 0, |f| {
        f.write_all("0123456789".as_bytes())
    });
    assert_eq!(res.ok(), Some(()));
    assert!(Path::new(&pbf).exists());

    // Cached
    assert!(cache.lookup("topic", 1, 2, 0, |_| Ok(())).is_ok());

    // Read from cache
    let mut s = String::new();
    cache.lookup("topic", 1, 2, 0, |f| {
        f.read_to_string(&mut s).map(|_| ())
    });
    assert_eq!(&s, "0123456789");
}
