//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::cache::Cache;
use crate::cache::s3cache::S3Cache;
use std::fs;
use std::path::Path;

#[test]
fn test_s3cache() {
    
    let cache = S3Cache {
        endpoint: "http://localhost:9000".to_string(),
        bucket_name: "trex".to_string(),
        access_key: "miniostorage".to_string(),
        secret_key: "miniostorage".to_string(),
        region: "my-region".to_string(),
        baseurl: Some("http://localhost:6767".to_string()),
    };
    let path = "tileset/0/1/2.pbf";
    let fullpath = format!("{}/{}", cache.bucket_name, path);
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
