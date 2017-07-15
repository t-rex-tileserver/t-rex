//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use toml;
use core::Config;
use core::config::GridCfg;
use core::grid::{Grid, Origin, Extent, ExtentInt};


#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TestGrid {
    pub srid: i32,
    pub origin: Origin,
}

#[test]
fn test_ser() {
    let grid = TestGrid {
        srid: 4236,
        origin: Origin::BottomLeft,
    };
    assert_eq!(toml::to_string(&grid),
               Ok("srid = 4236\norigin = \"BottomLeft\"\n".to_string()));
    let value = toml::Value::try_from(&grid);
    println!("{:?}", value);
    let toml = toml::from_str::<TestGrid>("srid = 4236\norigin = \"BottomLeft\"\n");
    println!("{:?}", toml);
    let value = "srid = 4236\norigin = \"BottomLeft\"\n"
        .parse::<toml::Value>()
        .unwrap();
    println!("{:?}", value);
    let toml2 = value.try_into::<TestGrid>();
    println!("{:?}", toml2);
    assert_eq!(toml2.unwrap(), grid);
    assert_eq!(toml.unwrap(), grid);
}


#[test]
fn test_bbox() {
    use std::u32;

    let grid = Grid::web_mercator();

    let extent000 = grid.tile_extent(0, 0, 0);
    assert_eq!(extent000,
               Extent {
                   minx: -20037508.342789248,
                   miny: -20037508.342789248,
                   maxx: 20037508.342789248,
                   maxy: 20037508.342789248,
               });

    let extent = grid.tile_extent_xyz(486, 332, 10);
    assert_eq!(extent,
               Extent {
                   minx: -1017529.7205322683,
                   miny: 7005300.768279828,
                   maxx: -978393.9620502591,
                   maxy: 7044436.526761841,
               });
    let extent = grid.tile_extent(486, 691, 10);
    assert_eq!(extent,
               Extent {
                   minx: -1017529.7205322683,
                   miny: 7005300.768279828,
                   maxx: -978393.9620502591,
                   maxy: 7044436.526761841,
               });

    //overflow
    let extent = grid.tile_extent_xyz(486, u32::MAX, 10);
    assert_eq!(extent,
               Extent {
                   minx: -1017529.7205322683,
                   miny: -20037508.342789248,
                   maxx: -978393.9620502591,
                   maxy: -19998372.58430724,
               });

    let extent_ch = grid.tile_extent_xyz(1073, 717, 11);
    assert_eq!(extent_ch,
               Extent {
                   minx: 958826.0828092434,
                   miny: 5987771.04774756,
                   maxx: 978393.9620502479,
                   maxy: 6007338.926988564,
               });

    let wgs84extent000 = Grid::wgs84().tile_extent(0, 0, 0);
    assert_eq!(wgs84extent000,
               Extent {
                   minx: -180.0,
                   miny: -90.0,
                   maxx: 0.0,
                   maxy: 90.0,
               });
}

#[test]
fn test_grid_calculations() {
    let grid = Grid::web_mercator();

    assert_eq!(grid.pixel_width(10), 152.8740565703525);
    assert_eq!(grid.scale_denominator(10), 545978.7734655447);

    assert_eq!(grid.level_limit(0), (1, 1));
    assert_eq!(grid.level_limit(10), (1024, 1024));

    let limits = grid.tile_limits(grid.tile_extent(0, 0, 0), 0);
    assert_eq!(limits[0],
               ExtentInt {
                   minx: 0,
                   miny: 0,
                   maxx: 1,
                   maxy: 1,
               });
    assert_eq!(limits[10],
               ExtentInt {
                   minx: 0,
                   miny: 0,
                   maxx: 1024,
                   maxy: 1024,
               });

    let limits = grid.tile_limits(Extent {
                                      minx: -1017529.7205322683,
                                      miny: 7005300.768279828,
                                      maxx: -978393.9620502591,
                                      maxy: 7044436.526761841,
                                  },
                                  0);
    assert_eq!(limits[0],
               ExtentInt {
                   minx: 0,
                   miny: 0,
                   maxx: 1,
                   maxy: 1,
               });
    assert_eq!(limits[10],
               ExtentInt {
                   minx: 486,
                   miny: 691,
                   maxx: 487,
                   maxy: 692,
               });

    let extent = grid.tile_extent(133, 165, 8);
    assert_eq!(extent, grid.tile_extent_xyz(133, 90, 8));
    assert_eq!(extent,
               Extent {
                   minx: 782715.1696402021,
                   miny: 5792092.25533751,
                   maxx: 939258.2035682425,
                   maxy: 5948635.289265554,
               });
    let limits = grid.tile_limits(extent, 0);
    assert_eq!(limits[8],
               ExtentInt {
                   minx: 133,
                   miny: 165,
                   maxx: 134,
                   maxy: 166,
               });
}

#[test]
fn test_grid_from_config() {
    use core::parse_config;

    let toml = r#"
        #[grid]
        predefined = "web_mercator"
        "#;
    let config: GridCfg = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(grid.extent,
               Extent {
                   minx: -20037508.3427892480,
                   miny: -20037508.3427892480,
                   maxx: 20037508.3427892480,
                   maxy: 20037508.3427892480,
               });

    let toml = r#"
        #[grid]
        width = 256
        height = 256
        extent = { minx = 2420000.0, miny = 1030000.0, maxx = 2900000.0, maxy = 1350000.0 }
        srid = 2056
        units = "M"
        resolutions = [4000.0,3750.0,3500.0,3250.0,3000.0,2750.0,2500.0,2250.0,2000.0,1750.0,1500.0,1250.0,1000.0,750.0,650.0,500.0,250.0,100.0,50.0,20.0,10.0,5.0,2.5,2.0,1.5,1.0,0.5,0.25,0.1]
        origin = "TopLeft"
        "#;
    let config: GridCfg = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(grid.extent,
               Extent {
                   minx: 2420000.0,
                   miny: 1030000.0,
                   maxx: 2900000.0,
                   maxy: 1350000.0,
               });
    assert_eq!(grid.origin, Origin::TopLeft);

    let extent = grid.tile_extent(10, 4, 17); // lake of Zurich
    assert_eq!(extent,
               Extent {
                   minx: 2676000.,
                   miny: 1222000.,
                   maxx: 2701600.,
                   maxy: 1247600.,
               });
    //BBOX ZH: (2669255.48 1223902.28, 2716899.60125 1283304.23625)
    let extent = grid.tile_extent_xyz(10, 4, 17);
    assert_eq!(extent,
               Extent {
                   minx: 2676000.,
                   miny: -109951160275600.,
                   maxx: 2701600.,
                   maxy: -109951160250000.,
               });
}


mod web_mercator {

    // --- Web Mercator calculations ---
    // Credits: Mercantile by Sean C. Gillies (https://github.com/mapbox/mercantile)

    use core::grid::Extent;
    use std::f64::consts;

    #[derive(PartialEq,Debug)]
    pub struct LngLat {
        pub lon: f64,
        pub lat: f64,
    }

    /// Returns the upper left (lon, lat) of a tile
    fn ul(xtile: u32, ytile: u32, zoom: u8) -> LngLat {
        let n = (zoom as f64).exp2();
        let lon_deg = xtile as f64 / n * 360.0 - 180.0;
        let lat_rad = (consts::PI * (1.0 - 2.0 * ytile as f64 / n))
            .sinh()
            .atan();
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
        let y = 6378137.0 *
                ((consts::PI * 0.25) + (0.5 * lat.to_radians()))
                    .tan()
                    .ln();
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
        assert_eq!(lnglat,
                   LngLat {
                       lon: -9.140625,
                       lat: 53.33087298301705,
                   });
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
        assert_eq!(extent,
                   Extent {
                       minx: -1017529.7205322663,
                       miny: 7005300.768279833,
                       maxx: -978393.962050256,
                       maxy: 7044436.526761846,
                   });
    }

    #[test]
    fn test_merc_tile_bounds() {
        let bbox = tile_bounds(486, 332, 10);
    #[cfg(not(target_os = "macos"))]
        assert_eq!(bbox,
                   Extent {
                       minx: -9.140625,
                       miny: 53.120405283106564,
                       maxx: -8.7890625,
                       maxy: 53.33087298301705,
                   });
    #[cfg(target_os = "macos")]
        assert_eq!(bbox,
                   Extent {
                       minx: -9.140625,
                       miny: 53.12040528310657,
                       maxx: -8.7890625,
                       maxy: 53.33087298301705,
                   });
    }

}
