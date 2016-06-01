//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::io::{Read,Write};
use std::io;


pub trait Cache {
    fn lookup<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut read: F) -> Result<(), io::Error>
        where F : FnMut(&mut Read) -> Result<(), io::Error>;
    fn store<F>(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16, mut write: F) -> Result<(), io::Error>
        where F : Fn(&mut Write) -> Result<(), io::Error>;
}
