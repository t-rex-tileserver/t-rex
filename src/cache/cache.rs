//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::io::Read;
use std::io;


pub trait Cache {
    fn read<F>(&self, path: &str, read: F) -> bool where F: FnMut(&mut Read);
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error>;
    fn exists(&self, path: &str) -> bool;
}


pub struct Nocache;

impl Cache for Nocache {
    #[allow(unused_variables)]
    fn read<F>(&self, path: &str, read: F) -> bool
        where F: FnMut(&mut Read)
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
