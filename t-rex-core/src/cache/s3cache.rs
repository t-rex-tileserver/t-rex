//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::cache::cache::Cache;
use rusoto_core::{Client, HttpClient, Region};
use rusoto_credential::StaticProvider;
use rusoto_s3::{
    GetObjectRequest, HeadObjectOutput, HeadObjectRequest, PutObjectError, PutObjectRequest,
    S3Client, S3,
};
use std::io::{self, Read};
use tokio::runtime::Runtime;

#[derive(Clone)]
pub struct S3Cache {
    pub endpoint: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub baseurl: Option<String>,
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
        let region = Region::Custom {
            name: self.region.to_string(),
            endpoint: self.endpoint.to_string(),
        };

        let client = S3Client::new_with_client(
            Client::new_with(
                StaticProvider::new(
                    self.access_key.to_string(),
                    self.secret_key.to_string(),
                    None,
                    None,
                ),
                HttpClient::new().expect("Could not instantiate a new http client??"),
            ),
            region.clone(),
        );

        let request = GetObjectRequest {
            bucket: self.bucket_name.to_string(),
            key: path.to_string(),
            ..Default::default()
        };
        let object = client.get_object(request);
        let mut rt = Runtime::new().unwrap();
        let get_object_result = rt.block_on(object);
        match get_object_result {
            Ok(result) => {
                read(&mut result.body.unwrap().into_blocking_read());
                true
            }
            Err(_) => false,
        }
    }
    fn write(&self, path: &str, obj: &[u8]) -> Result<(), io::Error> {
        let region = Region::Custom {
            name: self.region.to_string(),
            endpoint: self.endpoint.to_string(),
        };
        let client = S3Client::new_with_client(
            Client::new_with(
                StaticProvider::new(
                    self.access_key.to_string(),
                    self.secret_key.to_string(),
                    None,
                    None,
                ),
                HttpClient::new().expect("Could not instantiate a new http client??"),
            ),
            region.clone(),
        );
        let request = PutObjectRequest {
            bucket: self.bucket_name.to_owned(),
            key: path.to_owned(),
            body: Some(obj.to_vec().into()),
            ..Default::default()
        };
        let mut rt = Runtime::new().unwrap();
        let object = client.put_object(request);
        let put_object_result = rt.block_on(object);
        match put_object_result {
            Ok(result) => Ok(()),
            Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err.to_string())),
        }
    }
    fn exists(&self, path: &str) -> bool {
        println!("Arg! Something went wrong: {:?}", path);
        return true;
    }
}
