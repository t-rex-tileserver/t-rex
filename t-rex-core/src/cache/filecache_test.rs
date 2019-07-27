//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::cache::Cache;
use crate::cache::filecache::Filecache;
use std::fs;
use std::path::Path;

#[test]
fn test_dircache() {
    use std::env;

    let mut dir = env::temp_dir();
    dir.push("t_rex_test");
    let basepath = format!("{}", &dir.display());
    let _ = fs::remove_dir_all(&basepath);

    let cache = Filecache {
        basepath: basepath,
        baseurl: Some("http://localhost:6767".to_string()),
    };
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
