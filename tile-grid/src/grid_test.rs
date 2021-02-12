//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::grid::{extent_wgs84_to_merc, lonlat_to_merc, Extent, ExtentInt, Grid};

#[test]
fn test_bbox() {
    use std::u32;

    let grid = Grid::web_mercator();

    let extent000 = grid.tile_extent(0, 0, 0);
    assert_eq!(
        extent000,
        Extent {
            minx: -20037508.342789248,
            miny: -20037508.342789248,
            maxx: 20037508.342789248,
            maxy: 20037508.342789248,
        }
    );

    let extent = grid.tile_extent_xyz(486, 332, 10);
    assert_eq!(
        extent,
        Extent {
            minx: -1017529.7205322683,
            miny: 7005300.768279836,
            maxx: -978393.9620502554,
            maxy: 7044436.526761845
        }
    );

    let extent = grid.tile_extent(486, 691, 10);
    assert_eq!(
        extent,
        Extent {
            minx: -1017529.7205322683,
            miny: 7005300.768279836,
            maxx: -978393.9620502554,
            maxy: 7044436.526761845
        }
    );

    let extent = grid.tile_extent(32, 42, 6);
    assert_eq!(
        extent,
        Extent {
            minx: 0.0,
            miny: 6261721.357121639,
            maxx: 626172.1357121654,
            maxy: 6887893.492833804
        }
    );

    //overflow
    let extent = grid.tile_extent_xyz(486, u32::MAX, 10);
    assert_eq!(
        extent,
        Extent {
            minx: -1017529.7205322683,
            miny: -20037508.342789248,
            maxx: -978393.9620502554,
            maxy: -19998372.58430724
        }
    );

    let extent_ch = grid.tile_extent_xyz(1073, 717, 11);
    assert_eq!(
        extent_ch,
        Extent {
            minx: 958826.0828092508,
            miny: 5987771.047747567,
            maxx: 978393.9620502554,
            maxy: 6007338.926988572
        }
    );

    let wgs84extent000 = Grid::wgs84().tile_extent(0, 0, 0);
    assert_eq!(
        wgs84extent000,
        Extent {
            minx: -180.0,
            miny: -90.0,
            maxx: 0.0,
            maxy: 90.0,
        }
    );
}

#[test]
fn test_resolutions() {
    // Formula: http://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Resolution_and_Scale
    // const PIXEL_WIDTH_Z0: f64 = 2.0 * 6378137.0 * consts::PI / 256.0; //  = 40075016.68557849 / 256
    // Calculated pixel width without rounding results in non-symmetrical grid.tile_extent(0, 0, 0)
    const PIXEL_WIDTH_Z0: f64 = 156543.0339280410; // rounded to 10 digits
    let resolutions: Vec<f64> = (0..23)
        .map(|z| PIXEL_WIDTH_Z0 / (z as f64).exp2())
        .collect();

    let grid = Grid::web_mercator();
    let grid_resolutions: Vec<f64> = (0..23).map(|z| grid.pixel_width(z)).collect();

    assert_eq!(resolutions, grid_resolutions);
}

#[test]
fn test_grid_calculations() {
    let grid = Grid::web_mercator();

    assert_eq!(grid.pixel_width(10), 152.87405657035254);
    assert_eq!(grid.scale_denominator(10), 545978.7734655448);

    assert_eq!(grid.level_limit(0), (1, 1));
    assert_eq!(grid.level_limit(10), (1024, 1024));

    let limits = grid.tile_limits(grid.tile_extent(0, 0, 0), 0);
    assert_eq!(
        limits[0],
        ExtentInt {
            minx: 0,
            miny: 0,
            maxx: 1,
            maxy: 1,
        }
    );
    assert_eq!(
        limits[10],
        ExtentInt {
            minx: 0,
            miny: 0,
            maxx: 1024,
            maxy: 1024,
        }
    );

    let limits = grid.tile_limits(
        Extent {
            minx: -1017529.7205322683,
            miny: 7005300.768279828,
            maxx: -978393.9620502591,
            maxy: 7044436.526761841,
        },
        0,
    );
    assert_eq!(
        limits[0],
        ExtentInt {
            minx: 0,
            miny: 0,
            maxx: 1,
            maxy: 1,
        }
    );
    assert_eq!(
        limits[10],
        ExtentInt {
            minx: 486,
            miny: 691,
            maxx: 487,
            maxy: 692,
        }
    );

    let extent = grid.tile_extent(133, 165, 8);
    assert_eq!(extent, grid.tile_extent_xyz(133, 90, 8));
    assert_eq!(
        extent,
        Extent {
            minx: 782715.1696402058,
            miny: 5792092.255337518,
            maxx: 939258.2035682462,
            maxy: 5948635.289265558
        }
    );
    let limits = grid.tile_limits(extent, 0);
    assert_eq!(
        limits[8],
        ExtentInt {
            minx: 133,
            miny: 165,
            maxx: 134,
            maxy: 166,
        }
    );
}

#[test]
fn test_wgs84_grid() {
    let grid = Grid::wgs84();

    assert_eq!(grid.pixel_width(10), 76.43702828517625);
    assert_eq!(grid.scale_denominator(10), 272989.38673277234);
}

#[test]
fn test_projected_extent() {
    let extent_wgs84 = Extent {
        minx: 4.0,
        miny: 52.0,
        maxx: 5.0,
        maxy: 53.0,
    };
    #[cfg(not(target_os = "macos"))]
    let extent_3857 = Extent {
        minx: 445277.96317309426,
        miny: 6800125.454397307,
        maxx: 556597.4539663679,
        maxy: 6982997.920389788,
    };
    #[cfg(target_os = "macos")]
    let extent_3857 = Extent {
        minx: 445277.96317309426,
        miny: 6800125.454397305,
        maxx: 556597.4539663679,
        maxy: 6982997.920389788,
    };
    let projected = extent_wgs84_to_merc(&extent_wgs84);
    assert_eq!(projected, extent_3857);
    assert_eq!(
        lonlat_to_merc(extent_wgs84.minx, extent_wgs84.miny),
        (projected.minx, projected.miny)
    );
}

mod web_mercator {

    // --- Web Mercator calculations ---
    // Credits: Mercantile by Sean C. Gillies (https://github.com/mapbox/mercantile)

    use crate::grid::Extent;
    use std::f64::consts;

    #[derive(PartialEq, Debug)]
    pub struct LngLat {
        pub lon: f64,
        pub lat: f64,
    }

    /// Returns the upper left (lon, lat) of a tile
    fn ul(xtile: u32, ytile: u32, zoom: u8) -> LngLat {
        let n = (zoom as f64).exp2();
        let lon_deg = xtile as f64 / n * 360.0 - 180.0;
        let lat_rad = (consts::PI * (1.0 - 2.0 * ytile as f64 / n)).sinh().atan();
        let lat_deg = lat_rad.to_degrees();
        LngLat {
            lon: lon_deg,
            lat: lat_deg,
        }
    }

    /// Returns the Spherical Mercator (x, y) in meters
    fn xy(lon: f64, lat: f64) -> (f64, f64) {
        //lng, lat = truncate_lnglat(lng, lat)
        let x = 6378137.0 * lon.to_radians();
        let y = 6378137.0 * ((consts::PI * 0.25) + (0.5 * lat.to_radians())).tan().ln();
        (x, y)
    }

    /// Returns the Spherical Mercator bounding box of a tile
    fn tile_extent(xtile: u32, ytile: u32, zoom: u8) -> Extent {
        let a = ul(xtile, ytile, zoom);
        let (ax, ay) = xy(a.lon, a.lat);
        let b = ul(xtile + 1, ytile + 1, zoom);
        let (bx, by) = xy(b.lon, b.lat);
        Extent {
            minx: ax,
            miny: by,
            maxx: bx,
            maxy: ay,
        }
    }

    /// Returns the (lon, lat) bounding box of a tile
    fn tile_bounds(xtile: u32, ytile: u32, zoom: u8) -> Extent {
        let a = ul(xtile, ytile, zoom);
        let b = ul(xtile + 1, ytile + 1, zoom);
        Extent {
            minx: a.lon,
            miny: b.lat,
            maxx: b.lon,
            maxy: a.lat,
        }
    }

    #[test]
    fn test_ul() {
        let lnglat = ul(486, 332, 10);
        assert_eq!(
            lnglat,
            LngLat {
                lon: -9.140625,
                lat: 53.33087298301705,
            }
        );
        assert_eq!(
            ul(32, 42, 6),
            LngLat {
                lon: 0.0,
                lat: -48.92249926375824,
            }
        );
    }

    #[test]
    fn test_xy() {
        assert_eq!(xy(0.0, 0.0), (0.0, -0.0000000007081154551613622));
        let lnglat = ul(486, 332, 10);
        assert_eq!(
            xy(lnglat.lon, lnglat.lat),
            (-1017529.7205322663, 7044436.526761846)
        );
        let lnglat = ul(32, 42, 6);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(xy(lnglat.lon, lnglat.lat), (0.0, -6261721.357121639));
        #[cfg(target_os = "macos")]
        assert_eq!(xy(lnglat.lon, lnglat.lat), (0.0, -6261721.35712164));
    }

    #[test]
    fn test_merc_tile_extent() {
        let extent = tile_extent(486, 332, 10);
        assert_eq!(
            extent,
            Extent {
                minx: -1017529.7205322663,
                miny: 7005300.768279833,
                maxx: -978393.962050256,
                maxy: 7044436.526761846,
            }
        );
    }

    #[test]
    fn test_merc_tile_bounds() {
        let bbox = tile_bounds(486, 332, 10);
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            bbox,
            Extent {
                minx: -9.140625,
                miny: 53.120405283106564,
                maxx: -8.7890625,
                maxy: 53.33087298301705,
            }
        );
        #[cfg(target_os = "macos")]
        assert_eq!(
            bbox,
            Extent {
                minx: -9.140625,
                miny: 53.12040528310657,
                maxx: -8.7890625,
                maxy: 53.33087298301705,
            }
        );
    }
}
