//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::io::{Read,Write};
use std::io;


pub trait Cache {
    fn lookup<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>;
    fn store<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>;
}


pub struct Nocache;

impl Cache for Nocache {
     #[allow(unused_variables)]
    fn lookup<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>
    {
        Ok(())
    }
     #[allow(unused_variables)]
    fn store<F>(&self, tileset: &str, xtile: u16, ytile: u16, zoom: u16, write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>
    {
        Ok(())
    }
}
