//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use core::Config;
use toml;
use rustc_serialize::{Decodable, Decoder};


#[derive(PartialEq, RustcDecodable, Debug)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

/// Min and max grid cell numbers
#[derive(PartialEq,Debug)]
pub struct ExtentInt {
    pub minx: u16,
    pub miny: u16,
    pub maxx: u16,
    pub maxy: u16,
}

#[derive(PartialEq, Debug)]
pub enum Origin {
    TopLeft, BottomLeft //TopRight, BottomRight
}

impl Decodable for Origin {
    fn decode<D: Decoder>(d: &mut D) -> Result<Origin, D::Error> {
        let val = try!(d.read_str());
        match &val as &str {
            "TopLeft" => Ok(Origin::TopLeft),
            "BottomLeft" => Ok(Origin::BottomLeft),
            _ => Err(d.error(&*format!("Unknown value `{}`", val)))
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Unit {
    M, DD, Ft
}

impl Decodable for Unit {
    fn decode<D: Decoder>(d: &mut D) -> Result<Unit, D::Error> {
        let val = try!(d.read_str());
        match &val as &str {
            "M" => Ok(Unit::M),
            "DD" => Ok(Unit::DD),
            "Ft" => Ok(Unit::Ft),
            _ => Err(d.error(&*format!("Unknown value `{}`", val)))
        }
    }
}

// Credits: MapCache by Thomas Bonfort (http://mapserver.org/mapcache/)
#[derive(RustcDecodable, Debug)]
pub struct Grid {
    /// The width and height of an individual tile, in pixels.
    width: u16,
    height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.). Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    extent: Extent,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units
    pub units: Unit,
    /// This is a list of resolutions for each of the zoom levels defined by the grid. This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions are expressed in “units-per-pixel”, depending on the unit used by the grid (e.g. resolutions are in meters per pixel for most grids used in webmapping).
    resolutions: Vec<f64>,
    /// Grid origin
    origin: Origin,
}

impl Grid {
    /// WGS84 grid
    pub fn wgs84() -> Grid {
        Grid {
            width: 256, height: 256,
            extent: Extent {minx: -180.0, miny: -90.0, maxx: 180.0, maxy: 90.0},
            srid: 4236,
            units: Unit::DD,
            resolutions: vec![
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
            ],
            origin: Origin::BottomLeft
        }
    }

    /// Web Mercator grid (Google maps compatible)
    pub fn web_mercator() -> Grid {
        Grid {
            width: 256, height: 256,
            extent: Extent {minx: -20037508.3427892480, miny: -20037508.3427892480,
                            maxx: 20037508.3427892480, maxy: 20037508.3427892480},
            srid: 3857,
            units: Unit::M,
            resolutions: vec![
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
            ],
            origin: Origin::BottomLeft
        }
    }

    pub fn nlevels(&self) -> u8 {
        self.resolutions.len() as u8
    }
    pub fn pixel_width(&self, zoom: u8) -> f64 {
        self.resolutions[zoom as usize]  //TODO: assumes grid unit 'm'
    }
    pub fn scale_denominator(&self, zoom: u8) -> f64 {
        let pixel_screen_width = 0.0254 / 96.0; //FIXME: assumes 96dpi - check with mapnik
        self.pixel_width(zoom) / pixel_screen_width
    }
    /// Extent of a given tile in the grid given its x, y, and z
    pub fn tile_extent(&self, xtile: u16, ytile: u16, zoom: u8) -> Extent {
        // based on mapcache_grid_get_tile_extent
        let res = self.resolutions[zoom as usize];
        let tile_sx = self.width as f64;
        let tile_sy = self.height as f64;
        match self.origin {
            Origin::BottomLeft =>
                Extent {
                    minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                    miny: self.extent.miny + (res * ytile as f64 * tile_sy),
                    maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                    maxy: self.extent.miny + (res * (ytile + 1) as f64 * tile_sy),
                },
            Origin::TopLeft =>
                Extent {
                    minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                    miny: self.extent.maxy - (res * (ytile + 1) as f64 * tile_sy),
                    maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                    maxy: self.extent.maxy - (res * ytile as f64 * tile_sy)
                }
        }
    }
    /// Extent of a given tile in GoogleMaps XYZ adressing scheme
    pub fn tile_extent_reverse_y(&self, xtile: u16, ytile: u16, zoom: u8) -> Extent {
        let res = self.resolutions[zoom as usize];
        let unitheight = self.height as f64 * res;
        let maxy = ((self.extent.maxy-self.extent.minx- 0.01* unitheight)/unitheight).ceil() as u16;
        let y = maxy-ytile-1;
        self.tile_extent(xtile, y, zoom)
    }
    /// (maxx, maxy) of grid level
    pub fn level_limit(&self, zoom: u8) -> (u16, u16) {
        let res = self.resolutions[zoom as usize];
        let unitheight = self.height as f64 * res;
        let unitwidth = self.width as f64 * res;

        let maxy = ((self.extent.maxy-self.extent.miny - 0.01* unitheight)/unitheight).ceil() as u16;
        let maxx = ((self.extent.maxx-self.extent.miny - 0.01* unitwidth)/unitwidth).ceil() as u16;
        (maxx, maxy)
    }
    /// Tile index limits covering extent
    pub fn tile_limits(&self, extent: Extent, tolerance: i16) -> Vec<ExtentInt> {
      // Based on mapcache_grid_compute_limits
      const EPSILON: f64 = 0.0000001;
      let nlevels = self.resolutions.len() as u8;
      (0..nlevels).map(|i| {
        let res = self.resolutions[i as usize];
        let unitheight = self.height as f64 * res;
        let unitwidth = self.width as f64 * res;
        let (level_maxx, level_maxy) = self.level_limit(i);

        let (mut minx, mut maxx, mut miny, mut maxy) = match self.origin {
            Origin::BottomLeft =>
                (
                    (((extent.minx - self.extent.minx) / unitwidth  + EPSILON).floor() as i16) - tolerance,
                    (((extent.maxx - self.extent.minx) / unitwidth  - EPSILON).ceil()  as i16) + tolerance,
                    (((extent.miny - self.extent.miny) / unitheight + EPSILON).floor() as i16) - tolerance,
                    (((extent.maxy - self.extent.miny) / unitheight - EPSILON).ceil()  as i16) + tolerance,
                ),
            Origin::TopLeft =>
                (
                    (((extent.minx - self.extent.minx) / unitwidth  + EPSILON).floor() as i16) - tolerance,
                    (((extent.maxx - self.extent.minx) / unitwidth  - EPSILON).ceil()  as i16) + tolerance,
                    (((self.extent.maxy - extent.maxy) / unitheight + EPSILON).floor() as i16) - tolerance,
                    (((self.extent.maxy - extent.miny) / unitheight - EPSILON).ceil()  as i16) + tolerance,
                )
        };

        // to avoid requesting out-of-range tiles
        if minx < 0 { minx = 0; }
        if maxx > level_maxx as i16 { maxx = level_maxx as i16 };
        if miny < 0 { miny = 0 };
        if maxy > level_maxy as i16 { maxy = level_maxy as i16 };

        ExtentInt { minx: minx as u16, maxx: maxx as u16, miny: miny as u16, maxy: maxy as u16 }
      }).collect()
    }
}

impl Config<Grid> for Grid {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        if config.lookup("grid").is_none() {
            return Err("Missing configuration entry [grid]".to_string())
        }
        if let Some(predef) = config.lookup("grid.predefined") {
            predef.as_str().ok_or("grid.predefined entry is not a string".to_string())
                .and_then(|gridname| {
                    match gridname {
                        "wgs84" => Ok(Grid::wgs84()),
                        "web_mercator" => Ok(Grid::web_mercator()),
                        _ => Err(format!("Unkown grid '{}'", gridname))
                    }
            })
        } else {
            let gridcfg = config.lookup("grid").unwrap();
            let mut decoder = toml::Decoder::new(gridcfg.clone());
            let grid = Grid::decode(&mut decoder);
            grid.map_err(|e| format!("Error reading configuration - {}", e))
        }
    }
    fn gen_config() -> String {
        let toml = r#"
[grid]
# Predefined grids: web_mercator, wgs84
predefined = "web_mercator"
"#;
        toml.to_string()
    }
}


#[test]
fn test_bbox() {
    let grid = Grid::web_mercator();

    let extent000 = grid.tile_extent(0, 0, 0);
    assert_eq!(extent000, Extent {minx: -20037508.342789248, miny: -20037508.342789248, maxx: 20037508.342789248, maxy: 20037508.342789248});

    let extent = grid.tile_extent_reverse_y(486, 332, 10);
    assert_eq!(extent, Extent {minx: -1017529.7205322683, miny: 7005300.768279828, maxx: -978393.9620502591, maxy: 7044436.526761841});
    let extent = grid.tile_extent(486, 691, 10);
    assert_eq!(extent, Extent {minx: -1017529.7205322683, miny: 7005300.768279828, maxx: -978393.9620502591, maxy: 7044436.526761841});

    let extent_ch = grid.tile_extent_reverse_y(1073, 717, 11);
    assert_eq!(extent_ch, Extent { minx: 958826.0828092434, miny: 5987771.04774756, maxx: 978393.9620502479, maxy: 6007338.926988564 });

    let wgs84extent000 = Grid::wgs84().tile_extent(0, 0, 0);
    assert_eq!(wgs84extent000, Extent { minx: -180.0, miny: -90.0, maxx: 0.0, maxy: 90.0 });
}

#[test]
fn test_grid_calculations() {
    let grid = Grid::web_mercator();

    assert_eq!(grid.pixel_width(10), 152.8740565703525);
    assert_eq!(grid.scale_denominator(10), 577791.7098721985);

    assert_eq!(grid.level_limit(0), (1, 1));
    assert_eq!(grid.level_limit(10), (1024, 1024));

    let limits = grid.tile_limits(grid.tile_extent(0, 0, 0), 0);
    assert_eq!(limits[0], ExtentInt {minx: 0, miny: 0, maxx: 1, maxy: 1 });
    assert_eq!(limits[10], ExtentInt {minx: 0, miny: 0, maxx: 1024, maxy: 1024 });

    let limits = grid.tile_limits(Extent {minx: -1017529.7205322683, miny: 7005300.768279828, maxx: -978393.9620502591, maxy: 7044436.526761841}, 0);
    assert_eq!(limits[0], ExtentInt {minx: 0, miny: 0, maxx: 1, maxy: 1 });
    assert_eq!(limits[10], ExtentInt {minx: 486, miny: 691, maxx: 487, maxy: 692});
}

#[test]
fn test_grid_from_config() {
    use core::parse_config;

    let toml = r#"
        [grid]
        predefined = "web_mercator"
        "#;
    let config = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(grid.extent, Extent {minx: -20037508.3427892480, miny: -20037508.3427892480,
                                    maxx: 20037508.3427892480, maxy: 20037508.3427892480});

    let toml = r#"
        [grid]
        width = 256
        height = 256
        extent = { minx = 2420000.0, miny = 1030000.0, maxx = 2900000.0, maxy = 1350000.0 }
        srid = 2056
        units = "M"
        resolutions = [4000.0,3750.0,3500.0,3250.0,3000.0,2750.0,2500.0,2250.0,2000.0,1750.0,1500.0,1250.0,1000.0,750.0,650.0,500.0,250.0,100.0,50.0,20.0,10.0,5.0,2.5,2.0,1.5,1.0,0.5,0.25,0.1]
        origin = "TopLeft"
        "#;
    let config = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(grid.extent, Extent {minx: 2420000.0, miny: 1030000.0,
                                    maxx: 2900000.0, maxy: 1350000.0});
    assert_eq!(grid.origin, Origin::TopLeft);

    let extent = grid.tile_extent(10, 4, 17); // lake of Zurich
    assert_eq!(extent, Extent {minx: 2676000., miny: 1222000., maxx: 2701600., maxy: 1247600.});
    //BBOX ZH: (2669255.48 1223902.28, 2716899.60125 1283304.23625)
    let extent = grid.tile_extent_reverse_y(10, 4, 17);
    assert_eq!(extent, Extent {minx: 2676000., miny: -1675219600., maxx: 2701600., maxy: -1675194000.});
}


#[cfg(feature = "webmercator")]
mod WebMercator {

// --- Web Mercator calculations ---
// Credits: Mercantile by Sean C. Gillies (https://github.com/mapbox/mercantile)

use std::f64::consts;

#[derive(PartialEq,Debug)]
pub struct LngLat {
    pub lon: f64,
    pub lat: f64,
}

/// Returns the upper left (lon, lat) of a tile
fn ul(xtile: u16, ytile: u16, zoom: u8) -> LngLat {
    let n = (zoom as f64).exp2();
    let lon_deg = xtile as f64 / n * 360.0 - 180.0;
    let lat_rad = (consts::PI * (1.0_f64 - 2.0_f64 * ytile as f64 / n)).sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    LngLat {lon: lon_deg, lat: lat_deg}
}

/// Returns the Spherical Mercator (x, y) in meters
fn xy(lon: f64, lat: f64) -> (f64, f64) {
    //lng, lat = truncate_lnglat(lng, lat)
    let x = 6378137.0_f64 * lon.to_radians();
    let y = 6378137.0_f64 *
        ((consts::PI*0.25) + (0.5_f64 * lat.to_radians())).tan().ln();
    (x, y)
}

/// Returns the Spherical Mercator bounding box of a tile
fn tile_extent(xtile: u16, ytile: u16, zoom: u8) -> Extent {
    let a = ul(xtile, ytile, zoom);
    let (ax, ay) = xy(a.lon, a.lat);
    let b = ul(xtile+1, ytile+1, zoom);
    let (bx, by) = xy(b.lon, b.lat);
    Extent {minx: ax, miny: by, maxx: bx, maxy: ay}
}

/// Returns the (lon, lat) bounding box of a tile
fn tile_bounds(xtile: u16, ytile: u16, zoom: u8) -> Extent {
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

}
