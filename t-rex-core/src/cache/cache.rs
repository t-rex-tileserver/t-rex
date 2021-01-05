//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::io;
use std::io::Read;

pub trait Cache {
    fn info(&self) -> String;
    /// Base URL of tile cache server published in metadata
    fn baseurl(&self) -> String;
    fn read<F>(&self, path: &str, read: F) -> bool
    where
        F: FnMut(&mut dyn Read);
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error>;
    fn exists(&self, path: &str) -> bool;
}

#[derive(Clone)]
pub struct Nocache;

impl Cache for Nocache {
    fn info(&self) -> String {
        "No cache".to_string()
    }
    fn baseurl(&self) -> String {
        "http://localhost:6767".to_string()
    }
    #[allow(unused_variables)]
    fn read<F>(&self, path: &str, read: F) -> bool
    where
        F: FnMut(&mut dyn Read),
    {
        false
    }
    #[allow(unused_variables)]
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        Ok(())
    }

    fn exists(&self, _path: &str) -> bool {
        false
    }
}
