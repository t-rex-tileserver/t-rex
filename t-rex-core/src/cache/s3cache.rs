//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::cache::Cache;
use rusoto_core::{Client, HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{GetObjectRequest, HeadObjectRequest, PutObjectRequest, S3Client, S3};
use std::io::{self, Read};
use std::path::Path;

#[derive(Clone)]
pub struct S3Cache {
    baseurl: Option<String>,
    client: S3Client,
    endpoint: String,
    bucket_name: String,
    key_prefix: Option<String>,
}

impl S3Cache {
    pub fn new(
        endpoint: &str,
        bucket_name: &str,
        access_key: &str,
        secret_key: &str,
        region: &str,
        baseurl: Option<String>,
        key_prefix: Option<String>,
    ) -> S3Cache {
        let region_object = Region::Custom {
            name: region.to_string(),
            endpoint: endpoint.to_string(),
        };
        let client = S3Client::new_with_client(
            Client::new_with(
                StaticProvider::new(access_key.to_string(), secret_key.to_string(), None, None),
                HttpClient::new().expect("Could not instantiate a new http client??"),
            ),
            region_object.clone(),
        );
        S3Cache {
            client: client,
            baseurl: baseurl,
            endpoint: endpoint.to_string(),
            bucket_name: bucket_name.to_string(),
            key_prefix: key_prefix,
        }
    }

    fn key_prefix(&self) -> String {
        self.key_prefix.clone().unwrap_or("".to_string())
    }

    fn full_path(&self, path: &str) -> String {
        let key_prefix = &self.key_prefix();
        match Path::new(key_prefix).join(path).to_str() {
            None => String::new(),
            Some(result) => result.to_string(),
        }
    }
}

impl Cache for S3Cache {
    fn info(&self) -> String {
        format!("Tile cache s3: {}/{}", self.endpoint, self.bucket_name)
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
        let key = self.full_path(path);
        if key.is_empty() {
            return false;
        }
        let request = GetObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };
        let client = self.client.clone();
        let response = client.get_object(request).sync();
        match response {
            Ok(mut result) => {
                let body = result.body.take().expect("The object has no body");
                read(&mut body.into_blocking_read());
                true
            }
            Err(_) => false,
        }
    }

    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        let key = self.full_path(path);
        if key.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "failed to join key_prefix with path",
            ));
        }
        let request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            body: Some(obj.to_vec().into()),
            ..Default::default()
        };
        let response = self.client.put_object(request).sync();
        match response {
            Ok(_) => Ok(()),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }

    fn exists(&self, path: &str) -> bool {
        let key = self.full_path(path);
        if key.is_empty() {
            return false;
        }
        let request = HeadObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: key.to_owned(),
            ..Default::default()
        };
        let response = self.client.head_object(request).sync();
        match response {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
