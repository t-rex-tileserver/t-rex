use core::layer::Layer;
use core::feature::Feature;
use core::grid::Extent;
use core::geom::GeometryType;
use core::geom;
use core::screen;
use mvt::vector_tile;
use mvt::geom_to_proto::{EncodableGeom,CommandSequence};


pub struct Tile<'a> {
    pub mvt_tile: vector_tile::Tile,
    tile_size: u32,
    extent: &'a Extent,
    feature_id: u64,
}

// --- conversion of geometries into screen coordinates

trait ScreenGeom<T> {
    fn from_geom(extent: &Extent, tile_size: u32, geom: T) -> Self;
}

impl ScreenGeom<geom::Point> for screen::Point {
    fn from_geom(extent: &Extent, tile_size: u32, point: geom::Point) -> Self {
        let x_span = extent.maxx - extent.minx;
        let y_span = extent.maxy - extent.miny;
        screen::Point {
            x: ((point.x-extent.minx) * tile_size as f64 / x_span) as i32,
            y: ((point.y-extent.miny) * tile_size as f64 / y_span) as i32
        }
    }
}

#[test]
fn test_point_to_screen_coords() {
    //let zh_mercator = geom::Point::new(949398.0, 6002729.0);
    let zh_mercator = geom::Point::new(960000.0, 6002729.0);
    //let zh_wgs84 = postgis::Point::<WGS84>::new(47.3703149, 8.5285874);
    let tile_extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let screen_pt = screen::Point::from_geom(&tile_extent, 4096, zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    //assert_eq!(screen_pt.encode().0, &[9,490,6262]);
}


// --- Tile creation functions

impl<'a> Tile<'a> {
    pub fn new(extent: &Extent, tile_size: u32) -> Tile {
        let mvt_tile = vector_tile::Tile::new();
        Tile {mvt_tile: mvt_tile, tile_size: tile_size, extent: extent, feature_id: 0 }
    }

    pub fn new_layer(&mut self, layer: Layer) -> vector_tile::Tile_Layer {
        vector_tile::Tile_Layer::new()
    }

    pub fn new_feature(&self, feature: Feature) -> vector_tile::Tile_Feature {
        vector_tile::Tile_Feature::new()
    }

    pub fn encode_geom(&self, geom: geom::GeometryType) -> CommandSequence {
        let screen_geom = match geom {
            GeometryType::Point(p) => screen::Point::from_geom(&self.extent, self.tile_size, p),
            _ => panic!("Geometry type not implemented yet")
        };
        screen_geom.encode()
    }

    pub fn add_layer(&mut self, mvt_layer: vector_tile::Tile_Layer) {

    }
}
