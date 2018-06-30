//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//! Statistics collector

use stats::{MinMax, OnlineStats};
use std::collections::BTreeMap;
use std::fmt;

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

    assert_eq!(stats.results("Layer.layerx").mean, 0.0);
}
