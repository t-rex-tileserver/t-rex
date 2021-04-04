//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//
use crate::cache::cache::Cache;
use crate::cache::s3cache::S3Cache;
use curl::easy::Easy;
use std::env;
use std::str;

#[test]
#[ignore]
fn test_s3cache() {
    if env::var("S3TEST").is_err() {
        return;
    }

    let cache = S3Cache::new(
        "http://localhost:9000",
        "trex",
        "miniostorage",
        "miniostorage",
        "my-region",
        Some("http://localhost:6767".to_string()),
        None,
        None,
    );
    let path = "tileset/0/1/2.pbf";
    let obj = "01234567910";

    // Cache miss
    assert_eq!(cache.read(path, |_| {}), false);

    // Write into cache
    let e = cache.write(path, obj.as_bytes());

    match e {
        Err(e) => {
            println!("Error writing file {:?}", e.to_string());
        }
        Ok(_) => {
            println!("Writing file successful");
        }
    }
    assert!(cache.exists(&path));

    // Cache hit
    assert_eq!(cache.read(path, |_| {}), true);

    // Read from cache
    let mut s = String::new();
    cache.read(path, |f| {
        let _ = f.read_to_string(&mut s);
    });
    assert_eq!(&s, obj);

    // check if Content-Encoding header set by default
    let mut handle = Easy::new();
    let mut headers = Vec::new();
    handle
        .url("http://localhost:9000/trex/tileset/0/1/2.pbf")
        .unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .header_function(|header| {
                headers.push(str::from_utf8(header).unwrap().to_string());
                true
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    assert!(headers.contains(&"Content-Encoding: gzip\r\n".to_string()));

    // test key_prefix
    let cache_prefix = S3Cache::new(
        "http://localhost:9000",
        "trex",
        "miniostorage",
        "miniostorage",
        "my-region",
        Some("http://localhost:6767".to_string()),
        Some("my-prefix".to_string()),
        Some(false),
    );

    // Cache miss
    assert_eq!(cache_prefix.read(path, |_| {}), false);

    // Write into cache
    let e = cache_prefix.write(path, obj.as_bytes());

    match e {
        Err(e) => {
            println!("Error writing file {:?}", e.to_string());
        }
        Ok(_) => {
            println!("Writing file successful");
        }
    }
    assert!(cache_prefix.exists(&path));

    // Cache hit
    assert_eq!(cache_prefix.read(path, |_| {}), true);

    // Read from cache
    let mut s = String::new();
    cache_prefix.read(path, |f| {
        let _ = f.read_to_string(&mut s);
    });
    assert_eq!(&s, obj);

    // check if Content-Encoding header not set
    let mut handle = Easy::new();
    let mut headers = Vec::new();
    handle
        .url("http://localhost:9000/trex/my-prefix/tileset/0/1/2.pbf")
        .unwrap();
    {
        let mut transfer = handle.transfer();
        transfer
            .header_function(|header| {
                headers.push(str::from_utf8(header).unwrap().to_string());
                true
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    assert!(!headers.contains(&"Content-Encoding: gzip\r\n".to_string()));
}
