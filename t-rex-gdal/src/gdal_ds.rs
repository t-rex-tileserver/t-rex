//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::DatasourceInput;
use gdal;
use gdal::vector::Dataset;
use core::feature::{Feature, FeatureAttr};
use core::geom::{self, GeometryType};
use core::grid::Extent;
use core::grid::Grid;
use core::layer::Layer;
use std::path::Path;


pub struct GdalDatasource {
    pub path: String,
}

impl GdalDatasource {
    pub fn new(path: &str) -> GdalDatasource {
        GdalDatasource { path: path.to_string() }
    }
}

struct VectorFeature<'a>(&'a gdal::vector::Feature<'a>);


impl<'a> VectorFeature<'a> {
    pub fn new(feat: &'a gdal::vector::Feature<'a>) -> VectorFeature<'a> {
        VectorFeature(feat)
    }
}

impl<'a> Feature for VectorFeature<'a> {
    fn fid(&self) -> Option<u64> {
        None // TODO
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        let attrs = Vec::new();
        // TODO
        attrs
    }
    fn geometry(&self) -> Result<GeometryType, String> {
        let _ogrgeom = self.0.geometry();
        Ok(GeometryType::Point(geom::Point::new(960000.0, 6002729.0, Some(3857)))) //TODO
    }
}

impl DatasourceInput for GdalDatasource {
    fn retrieve_features<F>(&self,
                            _layer: &Layer,
                            _extent: &Extent,
                            _zoom: u8,
                            _grid: &Grid,
                            mut read: F)
        where F: FnMut(&Feature)
    {
        let mut dataset = Dataset::open(Path::new(&self.path)).unwrap();
        let layer = dataset.layer(0).unwrap();
        for feature in layer.features().take(10) {
            let feat = VectorFeature::new(&feature);
            read(&feat);
        }
    }
}



#[test]
fn test_gdal_api() {
    use std::path::Path;
    use gdal::vector::Dataset;

    let mut dataset = Dataset::open(Path::new("natural_earth.gpkg")).unwrap();
    let layer = dataset.layer(0).unwrap();
    let feature = layer.features().next().unwrap();
    let name_field = feature.field("NAME").unwrap();
    let geometry = feature.geometry();
    assert_eq!(name_field.to_string(), Some("Colonia del Sacramento".to_string()));
    assert_eq!(geometry.wkt().unwrap(), "POINT (-6438719.62282072 -4093437.71441017)".to_string());
}



#[test]
fn test_gdal_retrieve() {
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    let grid = Grid::web_mercator();
    let extent = Extent {
        minx: 821850.9,
        miny: 5909499.5,
        maxx: 860986.7,
        maxy: 5948635.3,
    };

    let ds = GdalDatasource::new("natural_earth.gpkg");
    let mut reccnt = 0;
    ds.retrieve_features(&layer, &extent, 10, &grid, |feat| {
        assert_eq!("Ok(Point(Point { x: 960000, y: 6002729, srid: Some(3857) }))",
                   &*format!("{:?}", feat.geometry()));
        assert_eq!(0, feat.attributes().len());
        assert_eq!(None, feat.fid());
        reccnt += 1;
    });
    assert_eq!(10, reccnt);
}
