//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use std::collections::HashMap;
use std::str;

pub struct StaticFiles {
    files: HashMap<&'static str, (&'static [u8], &'static str)>,
}

impl StaticFiles {
    pub fn init() -> StaticFiles {
        let mut static_files = StaticFiles {
            files: HashMap::new(),
        };
        static_files.add(
            "favicon.ico",
            include_bytes!("static/favicon.ico"),
            "image/x-icon",
        );
        static_files.add(
            "index.html",
            include_bytes!("static/index.html"),
            "text/html",
        );
        static_files.add(
            "viewer.js",
            include_bytes!("static/viewer.js"),
            "application/javascript",
        );
        static_files.add(
            "viewer.css",
            include_bytes!("static/viewer.css"),
            "text/css",
        );
        static_files.add(
            "maputnik.html",
            include_bytes!("static/maputnik.html"),
            "text/html",
        );
        static_files.add(
            "maputnik.js",
            include_bytes!("static/maputnik.js"),
            "application/javascript",
        );
        static_files.add(
            "img/logo-color.svg",
            include_bytes!("static/img/logo-color.svg"),
            "image/svg+xml",
        );
        static_files.add(
            "fonts/Roboto-Regular.ttf",
            include_bytes!("static/fonts/Roboto-Regular.ttf"),
            "font/ttf",
        );
        static_files.add(
            "fonts/Roboto-Medium.ttf",
            include_bytes!("static/fonts/Roboto-Medium.ttf"),
            "font/ttf",
        );
        static_files
    }
    fn add(&mut self, name: &'static str, data: &'static [u8], media_type: &'static str) {
        self.files.insert(name, (data, media_type));
    }
    pub fn content(&self, base: Option<&str>, name: String) -> Option<&(&[u8], &str)> {
        let mut key = if name == "" {
            "index.html".to_string()
        } else {
            name
        };
        if let Some(path) = base {
            key = format!("{}/{}", path, key);
        }
        self.files.get(&key as &str)
    }
}
