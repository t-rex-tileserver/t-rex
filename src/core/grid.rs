//https://github.com/andelf/rust-postgis
use postgis as geom;
use postgis::{SRID,WGS84};
use core::screen;
use std::f64::consts;


#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
pub enum EPSG_3857 {}

impl SRID for EPSG_3857 {
    fn as_srid() -> Option<i32> { Some(3857) }
}


#[derive(PartialEq,Debug)]
pub struct LngLat {
    pub lon: f64,
    pub lat: f64,
}

#[derive(PartialEq,Debug)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

impl Extent {
    /// Convert geometry to tile relative coordinates
    pub fn geom_in_tile_extent(&self, tile_size: u32, geom: geom::Point<EPSG_3857>) -> screen::Point {
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


// Credits: MapCache by Thomas Bonfort (http://mapserver.org/mapcache/)
pub struct Grid {
    /// The width and height of an individual tile, in pixels. Must be specified as positive integers separated by a space character.
    width: u16,
    height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.). Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    extent: Extent,
    //srs: SRID,
    //units: m/dd/ft
    /// This is a list of resolutions for each of the zoom levels defined by the grid. This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions are expressed in “units-per-pixel”, depending on the unit used by the grid (e.g. resolutions are in meters per pixel for most grids used in webmapping).
    resolutions: &'static[f64],
    //origin: top-left, bottom-left, top-right and bottom-right
}

impl Grid {
    /// WGS84 grid
    pub fn wgs84() -> Grid {
        static WGS84_RESOLUTIONS: [f64; 18] = [
            0.703125000000000,
            0.351562500000000,
            0.175781250000000,
            8.78906250000000e-2,
            4.39453125000000e-2,
            2.19726562500000e-2,
            1.09863281250000e-2,
            5.49316406250000e-3,
            2.74658203125000e-3,
            1.37329101562500e-3,
            6.86645507812500e-4,
            3.43322753906250e-4,
            1.71661376953125e-4,
            8.58306884765625e-5,
            4.29153442382812e-5,
            2.14576721191406e-5,
            1.07288360595703e-5,
            5.36441802978516e-6
        ];

        Grid {
            width: 256, height: 256,
            extent: Extent {minx: -180.0, miny: -90.0, maxx: 180.0, maxy: 90.0},
            resolutions: &WGS84_RESOLUTIONS
        }
    }

    /// Web Mercator grid (Google maps compatible)
    pub fn web_mercator() -> Grid {
        static GOOGLE_RESOLUTIONS: [f64; 19] = [
            156543.0339280410,
            78271.51696402048,
            39135.75848201023,
            19567.87924100512,
            9783.939620502561,
            4891.969810251280,
            2445.984905125640,
            1222.992452562820,
            611.4962262814100,
            305.7481131407048,
            152.8740565703525,
            76.43702828517624,
            38.21851414258813,
            19.10925707129406,
            9.554628535647032,
            4.777314267823516,
            2.388657133911758,
            1.194328566955879,
            0.5971642834779395
        ];

        Grid {
            width: 256, height: 256,
            extent: Extent {minx: -20037508.3427892480, miny: -20037508.3427892480,
                            maxx: 20037508.3427892480, maxy: 20037508.3427892480},
            resolutions: &GOOGLE_RESOLUTIONS
        }
    }

    /// Extent of a given tile in the grid given its x, y, and z
    pub fn tile_extent(&self, xtile: u16, ytile: u16, zoom: u16) -> Extent {
        let res = self.resolutions[zoom as usize];
        let tile_sx = self.width as f64;
        let tile_sy = self.height as f64;
        Extent {
            minx: self.extent.minx + (res * xtile as f64 * tile_sx),
            miny: self.extent.miny + (res * ytile as f64 * tile_sy),
            maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
            maxy: self.extent.miny + (res * (ytile + 1) as f64 * tile_sy),
        }
        /* ORIGIN_TOP_LEFT:
            minx: self.extent.minx + (res * xtile as f64 * tile_sx),
            miny: self.extent.maxy - (res * (ytile + 1) as f64 * tile_sy),
            maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
            maxy: self.extent.maxy - (res * ytile as f64 * tile_sy)
        */
    }
    /// Extent of a given tile in GoogleMaps XYZ adressing scheme
    pub fn tile_extent_xyz(&self, xtile: u16, ytile: u16, zoom: u16) -> Extent {
        let res = self.resolutions[zoom as usize];
        let unitheight = self.height as f64 * res;
        let maxy = ((self.extent.maxy-self.extent.minx- 0.01* unitheight)/unitheight).ceil() as u16;
        let y = maxy-ytile-1;
        self.tile_extent(xtile, y, zoom)
    }
}

#[test]
fn test_bbox() {
    let grid = Grid::web_mercator();

    let extent000 = grid.tile_extent(0, 0, 0);
    assert_eq!(extent000, Extent {minx: -20037508.342789248, miny: -20037508.342789248, maxx: 20037508.342789248, maxy: 20037508.342789248});

    let extent = grid.tile_extent_xyz(486, 332, 10);
    assert_eq!(extent, Extent {minx: -1017529.7205322683, miny: 7005300.768279828, maxx: -978393.9620502591, maxy: 7044436.526761841});

    let wgs84extent000 = Grid::wgs84().tile_extent(0, 0, 0);
    assert_eq!(wgs84extent000, Extent { minx: -180.0, miny: -90.0, maxx: 0.0, maxy: 90.0 });
}


// --- Web Mercator calculations ---
// Credits: Mercantile by Sean C. Gillies (https://github.com/mapbox/mercantile)

/// Returns the upper left (lon, lat) of a tile
#[allow(dead_code)]
fn ul(xtile: u16, ytile: u16, zoom: u16) -> LngLat {
    let n = (zoom as f64).exp2();
    let lon_deg = xtile as f64 / n * 360.0 - 180.0;
    let lat_rad = (consts::PI * (1.0_f64 - 2.0_f64 * ytile as f64 / n)).sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    LngLat {lon: lon_deg, lat: lat_deg}
}

/// Returns the Spherical Mercator (x, y) in meters
#[allow(dead_code)]
fn xy(lon: f64, lat: f64) -> (f64, f64) {
    //lng, lat = truncate_lnglat(lng, lat)
    let x = 6378137.0_f64 * lon.to_radians();
    let y = 6378137.0_f64 *
        ((consts::PI*0.25) + (0.5_f64 * lat.to_radians())).tan().ln();
    (x, y)
}

/// Returns the Spherical Mercator bounding box of a tile
#[allow(dead_code)]
fn tile_extent(xtile: u16, ytile: u16, zoom: u16) -> Extent {
    let a = ul(xtile, ytile, zoom);
    let (ax, ay) = xy(a.lon, a.lat);
    let b = ul(xtile+1, ytile+1, zoom);
    let (bx, by) = xy(b.lon, b.lat);
    Extent {minx: ax, miny: by, maxx: bx, maxy: ay}
}

/// Returns the (lon, lat) bounding box of a tile
#[allow(dead_code)]
fn tile_bounds(xtile: u16, ytile: u16, zoom: u16) -> Extent {
    let a = ul(xtile, ytile, zoom);
    let b = ul(xtile+1, ytile+1, zoom);
    Extent {minx: a.lon, miny: b.lat, maxx: b.lon, maxy: a.lat}
}

#[test]
fn test_ul() {
    let lnglat = ul(486, 332, 10);
    assert_eq!(lnglat, LngLat {lon: -9.140625, lat: 53.33087298301705});
}

#[test]
fn test_xy() {
    let ul = ul(486, 332, 10);
    let xy_ = xy(ul.lon, ul.lat);
    assert_eq!(xy_, (-1017529.7205322663, 7044436.526761846));
    assert_eq!(xy(0.0, 0.0), (0.0, -0.0000000007081154551613622));
}

#[test]
fn test_merc_tile_extent() {
    let extent = tile_extent(486, 332, 10);
    assert_eq!(extent, Extent {minx: -1017529.7205322663, miny: 7005300.768279833, maxx: -978393.962050256, maxy: 7044436.526761846});
}

#[test]
fn test_merc_tile_bounds() {
    let bbox = tile_bounds(486, 332, 10);
    assert_eq!(bbox, Extent {minx: -9.140625, miny: 53.120405283106564, maxx: -8.7890625, maxy: 53.33087298301705});
}
