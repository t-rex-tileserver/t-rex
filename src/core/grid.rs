//https://github.com/andelf/rust-postgis
use postgis as geom;
use postgis::{SRID,WGS84};
use core::screen;


#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
pub enum EPSG_3857 {}

impl SRID for EPSG_3857 {
    fn as_srid() -> Option<i32> { Some(3857) }
}


pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64
}

impl Extent {
    /// Convert geometry to tile relative coordinates
    fn geom_in_tile_extent(&self, tile_size: u32, geom: geom::Point<EPSG_3857>) -> screen::Point {
        let x_span = self.maxx - self.minx;
        let y_span = self.maxy - self.miny;
        screen::Point {
            x: ((geom.x-self.minx) * tile_size as f64 / x_span) as i32,
            y: ((geom.y-self.miny) * tile_size as f64 / y_span) as i32 }
    }
}

#[test]
fn test_geom_in_tile_extent() {
    //let zh_mercator = geom::Point::<EPSG_3857>::new(949398.0, 6002729.0);
    let zh_mercator = geom::Point::<EPSG_3857>::new(960000.0, 6002729.0);
    //let zh_wgs84 = geom::Point::<WGS84>::new(47.3703149, 8.5285874);
    let tile_extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    let screen_pt = tile_extent.geom_in_tile_extent(
        4096, zh_mercator);
    assert_eq!(screen_pt, screen::Point { x: 245, y: 3131 });
    //assert_eq!(screen_pt.encode().0, &[9,490,6262]);
}
