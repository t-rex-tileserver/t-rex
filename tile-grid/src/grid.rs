//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

//!Tile grids

use std::f64::consts;

/// Geographic extent
#[derive(PartialEq, Clone, Debug)]
pub struct Extent {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

/// Min and max grid cell numbers
#[derive(PartialEq, Clone, Debug)]
pub struct ExtentInt {
    pub minx: u32,
    pub miny: u32,
    pub maxx: u32,
    pub maxy: u32,
}

// Max grid cell numbers
type CellIndex = (u32, u32);

/// Grid origin
#[derive(PartialEq, Clone, Debug)]
pub enum Origin {
    TopLeft,
    BottomLeft, //TopRight, BottomRight
}

/// Grid units
#[derive(PartialEq, Clone, Debug)]
pub enum Unit {
    Meters,
    Degrees,
    Feet,
}

/// Tile grid
#[derive(Clone, Debug)]
pub struct Grid {
    /// The width of an individual tile, in pixels.
    width: u16,
    /// The height of an individual tile, in pixels.
    height: u16,
    /// The geographical extent covered by the grid, in ground units (e.g. meters, degrees, feet, etc.).
    /// Must be specified as 4 floating point numbers ordered as minx, miny, maxx, maxy.
    /// The (minx,miny) point defines the origin of the grid, i.e. the pixel at the bottom left of the
    /// bottom-left most tile is always placed on the (minx,miny) geographical point.
    /// The (maxx,maxy) point is used to determine how many tiles there are for each zoom level.
    pub extent: Extent,
    /// Spatial reference system (PostGIS SRID).
    pub srid: i32,
    /// Grid units
    pub units: Unit,
    /// This is a list of resolutions for each of the zoom levels defined by the grid.
    /// This must be supplied as a list of positive floating point values, ordered from largest to smallest.
    /// The largest value will correspond to the grid’s zoom level 0. Resolutions
    /// are expressed in “units-per-pixel”,
    /// depending on the unit used by the grid (e.g. resolutions are in meters per
    /// pixel for most grids used in webmapping).
    resolutions: Vec<f64>,
    /// maxx/maxy for each resolution
    level_max: Vec<CellIndex>,
    /// Grid origin
    pub origin: Origin,
}

impl Grid {
    /// WGS84 grid
    pub fn wgs84() -> Grid {
        Grid::new(
            256,
            256,
            Extent {
                minx: -180.0,
                miny: -90.0,
                maxx: 180.0,
                maxy: 90.0,
            },
            4326,
            Unit::Degrees,
            vec![
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
                5.36441802978516e-6,
            ],
            Origin::BottomLeft,
        )
    }

    /// Web Mercator grid (Google maps compatible)
    pub fn web_mercator() -> Grid {
        Grid::new(
            256,
            256,
            Extent {
                minx: -20037508.3427892480,
                miny: -20037508.3427892480,
                maxx: 20037508.3427892480,
                maxy: 20037508.3427892480,
            },
            3857,
            Unit::Meters,
            // for calculation see fn test_resolutions
            vec![
                156543.0339280410,
                78271.5169640205,
                39135.75848201025,
                19567.879241005125,
                9783.939620502562,
                4891.969810251281,
                2445.9849051256406,
                1222.9924525628203,
                611.4962262814101,
                305.7481131407051,
                152.87405657035254,
                76.43702828517627,
                38.218514142588134,
                19.109257071294067,
                9.554628535647034,
                4.777314267823517,
                2.3886571339117584,
                1.1943285669558792,
                0.5971642834779396,
                0.2985821417389698,
                0.1492910708694849,
                0.07464553543474245,
                0.037322767717371225,
            ],
            Origin::BottomLeft,
        )
    }

    pub fn new(
        width: u16,
        height: u16,
        extent: Extent,
        srid: i32,
        units: Unit,
        resolutions: Vec<f64>,
        origin: Origin,
    ) -> Grid {
        let mut grid = Grid {
            width,
            height,
            extent,
            srid,
            units,
            resolutions,
            origin,
            level_max: Vec::new(),
        };
        grid.level_max = grid.level_max();
        grid
    }
    pub fn nlevels(&self) -> u8 {
        self.resolutions.len() as u8
    }
    pub fn maxzoom(&self) -> u8 {
        self.nlevels() - 1
    }
    /// Pixel width for 256x256 tile
    pub fn pixel_width(&self, zoom: u8) -> f64 {
        const METERS_PER_DEGREE: f64 = 6378137.0 * 2.0 * consts::PI / 360.0;
        match self.units {
            Unit::Meters => self.resolutions[zoom as usize],
            Unit::Degrees => self.resolutions[zoom as usize] * METERS_PER_DEGREE,
            Unit::Feet => self.resolutions[zoom as usize] * 0.3048,
        }
    }
    /// Scale denominator based on standardized pixel size (https://www.ogc.org/standards/se)
    pub fn scale_denominator(&self, zoom: u8) -> f64 {
        // Standardized rendering pixel size according to OGC Symbology Encoding standard
        const PIXEL_SCREEN_WIDTH: f64 = 0.00028;
        self.pixel_width(zoom) / PIXEL_SCREEN_WIDTH
    }
    /// Extent of a given tile in the grid given its x, y, and z in TMS adressing scheme
    pub fn tile_extent(&self, xtile: u32, ytile: u32, zoom: u8) -> Extent {
        // based on mapcache_grid_get_tile_extent
        let res = self.resolutions[zoom as usize];
        let tile_sx = self.width as f64;
        let tile_sy = self.height as f64;
        match self.origin {
            Origin::BottomLeft => Extent {
                minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                miny: self.extent.miny + (res * ytile as f64 * tile_sy),
                maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                maxy: self.extent.miny + (res * (ytile + 1) as f64 * tile_sy),
            },
            Origin::TopLeft => Extent {
                minx: self.extent.minx + (res * xtile as f64 * tile_sx),
                miny: self.extent.maxy - (res * (ytile + 1) as f64 * tile_sy),
                maxx: self.extent.minx + (res * (xtile + 1) as f64 * tile_sx),
                maxy: self.extent.maxy - (res * ytile as f64 * tile_sy),
            },
        }
    }
    /// reverse y tile for XYZ adressing scheme
    pub fn ytile_from_xyz(&self, ytile: u32, zoom: u8) -> u32 {
        // y = maxy-ytile-1
        let maxy = self.level_max[zoom as usize].1;
        let y = maxy.saturating_sub(ytile).saturating_sub(1);
        y
    }
    /// Extent of a given tile in XYZ adressing scheme
    pub fn tile_extent_xyz(&self, xtile: u32, ytile: u32, zoom: u8) -> Extent {
        let y = self.ytile_from_xyz(ytile, zoom);
        self.tile_extent(xtile, y, zoom)
    }
    /// (maxx, maxy) of grid level
    pub(crate) fn level_limit(&self, zoom: u8) -> CellIndex {
        let res = self.resolutions[zoom as usize];
        let unitheight = self.height as f64 * res;
        let unitwidth = self.width as f64 * res;

        let maxy =
            ((self.extent.maxy - self.extent.miny - 0.01 * unitheight) / unitheight).ceil() as u32;
        let maxx =
            ((self.extent.maxx - self.extent.minx - 0.01 * unitwidth) / unitwidth).ceil() as u32;
        (maxx, maxy)
    }
    /// (maxx, maxy) of all grid levels
    fn level_max(&self) -> Vec<CellIndex> {
        (0..self.nlevels())
            .map(|zoom| self.level_limit(zoom))
            .collect()
    }
    /// Tile index limits covering extent
    pub fn tile_limits(&self, extent: Extent, tolerance: i32) -> Vec<ExtentInt> {
        // Based on mapcache_grid_compute_limits
        const EPSILON: f64 = 0.0000001;
        (0..self.nlevels())
            .map(|i| {
                let res = self.resolutions[i as usize];
                let unitheight = self.height as f64 * res;
                let unitwidth = self.width as f64 * res;
                let (level_maxx, level_maxy) = self.level_max[i as usize];

                let (mut minx, mut maxx, mut miny, mut maxy) = match self.origin {
                    Origin::BottomLeft => (
                        (((extent.minx - self.extent.minx) / unitwidth + EPSILON).floor() as i32)
                            - tolerance,
                        (((extent.maxx - self.extent.minx) / unitwidth - EPSILON).ceil() as i32)
                            + tolerance,
                        (((extent.miny - self.extent.miny) / unitheight + EPSILON).floor() as i32)
                            - tolerance,
                        (((extent.maxy - self.extent.miny) / unitheight - EPSILON).ceil() as i32)
                            + tolerance,
                    ),
                    Origin::TopLeft => (
                        (((extent.minx - self.extent.minx) / unitwidth + EPSILON).floor() as i32)
                            - tolerance,
                        (((extent.maxx - self.extent.minx) / unitwidth - EPSILON).ceil() as i32)
                            + tolerance,
                        (((self.extent.maxy - extent.maxy) / unitheight + EPSILON).floor() as i32)
                            - tolerance,
                        (((self.extent.maxy - extent.miny) / unitheight - EPSILON).ceil() as i32)
                            + tolerance,
                    ),
                };

                // to avoid requesting out-of-range tiles
                if minx < 0 {
                    minx = 0;
                }
                if maxx > level_maxx as i32 {
                    maxx = level_maxx as i32
                };
                if miny < 0 {
                    miny = 0
                };
                if maxy > level_maxy as i32 {
                    maxy = level_maxy as i32
                };

                ExtentInt {
                    minx: minx as u32,
                    maxx: maxx as u32,
                    miny: miny as u32,
                    maxy: maxy as u32,
                }
            })
            .collect()
    }
}

/// Returns the Spherical Mercator (x, y) in meters
pub fn lonlat_to_merc(lon: f64, lat: f64) -> (f64, f64) {
    // from mod web_mercator in grid_test
    //lng, lat = truncate_lnglat(lng, lat)
    let x = 6378137.0 * lon.to_radians();
    let y = 6378137.0 * ((consts::PI * 0.25) + (0.5 * lat.to_radians())).tan().ln();
    (x, y)
}

/// Projected extent
pub fn extent_wgs84_to_merc(extent: &Extent) -> Extent {
    let (minx, miny) = lonlat_to_merc(extent.minx, extent.miny);
    let (maxx, maxy) = lonlat_to_merc(extent.maxx, extent.maxy);
    Extent {
        minx,
        miny,
        maxx,
        maxy,
    }
}
