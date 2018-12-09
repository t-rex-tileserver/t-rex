//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! Statistics collector

use serde_json;
use stats::{MinMax, OnlineStats};
use std::collections::BTreeMap;
use std::fmt;

type JsonResult = Result<serde_json::Value, serde_json::error::Error>;

type MeasurementType = u64;

struct StatCollector {
    online: OnlineStats,
    minmax: MinMax<MeasurementType>,
}

pub struct StatResults {
    pub len: usize,
    pub min: MeasurementType,
    pub max: MeasurementType,
    pub mean: f64,
    pub stddev: f64,
    pub variance: f64,
}

pub struct Statistics(BTreeMap<String, StatCollector>);

impl Statistics {
    pub fn new() -> Statistics {
        Statistics(BTreeMap::new())
    }
    fn collector(&mut self, key: String) -> &mut StatCollector {
        self.0.entry(key.to_string()).or_insert(StatCollector {
            online: OnlineStats::new(),
            minmax: MinMax::new(),
        })
    }
    pub fn add(&mut self, key: String, value: MeasurementType) {
        let collector = self.collector(key);
        collector.online.add(value);
        collector.minmax.add(value);
    }
    /// Return the current results.
    pub fn results(&self, key: &str) -> StatResults {
        if let Some(collector) = self.0.get(key) {
            StatResults {
                len: collector.minmax.len(),
                min: *collector.minmax.min().unwrap_or(&0),
                max: *collector.minmax.max().unwrap_or(&0),
                mean: collector.online.mean(),
                stddev: collector.online.stddev(),
                variance: collector.online.variance(),
            }
        } else {
            StatResults {
                len: 0,
                min: 0,
                max: 0,
                mean: 0.0,
                stddev: 0.0,
                variance: 0.0,
            }
        }
    }
    pub fn as_csv(&self) -> String {
        let mut lines = Vec::new();
        let mut header: Vec<String> = vec!["count", "min", "max", "mean", "stddev"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let maxkeylen = self.0.keys().map(|k| k.split('.').count()).max().unwrap();
        header.extend((0..maxkeylen).map(|n| format!("key{}", n)));
        lines.push(header.join(","));
        for key in self.0.keys() {
            let vals = self.results(&key);
            let mut cols = vec![
                vals.len.to_string(),
                vals.min.to_string(),
                vals.max.to_string(),
                vals.mean.to_string(),
                vals.stddev.to_string(),
            ];
            cols.extend(key.split('.').map(|k| k.to_string()));
            lines.push(cols.join(","));
        }
        lines.join("\n") + "\n"
    }
    pub fn as_json(&self) -> JsonResult {
        let json: Vec<serde_json::Value> = self
            .0
            .keys()
            .map(|key| {
                let vals = self.results(&key);
                let mut rec = json!({
                "key": key,
                "count": vals.len,
                "min": vals.min,
                "max": vals.max,
                "mean": vals.mean,
                "stddev": vals.stddev
                });
                for (n, k) in key.split('.').enumerate() {
                    rec.as_object_mut()
                        .unwrap()
                        .insert(format!("key{}", n), json!(k));
                }
                rec
            })
            .collect();
        Ok(json!(json))
    }
}

impl fmt::Debug for StatResults {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#measurements: {}, min: {}, max: {}, mean: {:.10} +/- {:.10}",
            self.len, self.min, self.max, self.mean, self.stddev
        )
    }
}

impl fmt::Debug for Statistics {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for key in self.0.keys() {
            let res = self.results(&key);
            let _ = write!(f, "{}: {:?}\n", key, res);
        }
        Ok(())
    }
}

#[test]
fn usage() {
    let mut stats = Statistics::new();
    stats.add("Layer.layer1".to_string(), 1);
    assert_eq!(stats.results("Layer.layer1").mean, 1.0);
    stats.add("Layer.layer1".to_string(), 2);
    assert_eq!(stats.results("Layer.layer1").mean, 1.5);
    stats.add("Layer.layer2".to_string(), 2);
    assert_eq!(stats.results("Layer.layer2").mean, 2.0);
    stats.add("Layer.layer1".to_string(), 3);
    assert_eq!(stats.results("Layer.layer1").mean, 2.0);
    assert_eq!(stats.results("Layer.layer1").stddev, 0.816496580927726);
    assert_eq!(stats.results("Layer.layer1").variance, 0.6666666666666666);
    assert_eq!(stats.results("Layer.layer1").len, 3);
    assert_eq!(stats.results("Layer.layer1").min, 1);
    assert_eq!(stats.results("Layer.layer1").max, 3);
    assert_eq!(&stats.as_csv(), "count,min,max,mean,stddev,key0,key1\n3,1,3,2,0.816496580927726,Layer,layer1\n1,2,2,2,0,Layer,layer2\n");
    let jsonstats = format!("{:#}", stats.as_json().unwrap());
    let expected = r#"[
  {
    "count": 3,
    "key": "Layer.layer1",
    "key0": "Layer",
    "key1": "layer1",
    "max": 3,
    "mean": 2.0,
    "min": 1,
    "stddev": 0.816496580927726
  },
  {
    "count": 1,
    "key": "Layer.layer2",
    "key0": "Layer",
    "key1": "layer2",
    "max": 2,
    "mean": 2.0,
    "min": 2,
    "stddev": 0.0
  }
]"#;
    println!("{}", jsonstats);
    assert_eq!(jsonstats, expected);

    assert_eq!(stats.results("Layer.layerx").mean, 0.0);
}
