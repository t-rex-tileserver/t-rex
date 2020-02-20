//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::cache::Cache;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Clone)]
pub struct Filecache {
    pub basepath: String,
    pub baseurl: Option<String>,
}

impl Cache for Filecache {
    fn info(&self) -> String {
        format!("Tile cache directory: {}", self.basepath)
    }
    fn baseurl(&self) -> String {
        self.baseurl
            .clone()
            .unwrap_or("http://localhost:6767".to_string())
    }
    fn read<F>(&self, path: &str, mut read: F) -> bool
    where
        F: FnMut(&mut dyn Read),
    {
        let fullpath = format!("{}/{}", self.basepath, path);
        debug!("Filecache.read {}", fullpath);
        match File::open(&fullpath) {
            Ok(mut f) => {
                read(&mut f);
                true
            }
            Err(_e) => false,
        }
    }
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        let fullpath = format!("{}/{}", self.basepath, path);
        debug!("Filecache.write {}", fullpath);
        let p = Path::new(&fullpath);
        fs::create_dir_all(p.parent().unwrap())?;
        let mut f = File::create(&fullpath)?;
        f.write_all(obj)
    }

    fn exists(&self, path: &str) -> bool {
        let fullpath = format!("{}/{}", self.basepath, path);
        Path::new(&fullpath).exists()
    }
}
