//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::GridCfg;

use crate::core::Config;
use tile_grid::{Extent, Grid, Origin};

#[test]
fn test_grid_from_config() {
    use crate::core::parse_config;

    let toml = r#"
        #[grid]
        predefined = "web_mercator"
        "#;
    let config: GridCfg = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(
        grid.extent,
        Extent {
            minx: -20037508.3427892480,
            miny: -20037508.3427892480,
            maxx: 20037508.3427892480,
            maxy: 20037508.3427892480,
        }
    );

    let toml = r#"
        #[grid.user]
        [user]
        width = 256
        height = 256
        extent = { minx = 2420000.0, miny = 1030000.0, maxx = 2900000.0, maxy = 1350000.0 }
        srid = 2056
        units = "m"
        resolutions = [4000.0,3750.0,3500.0,3250.0,3000.0,2750.0,2500.0,2250.0,2000.0,1750.0,1500.0,1250.0,1000.0,750.0,650.0,500.0,250.0,100.0,50.0,20.0,10.0,5.0,2.5,2.0,1.5,1.0,0.5,0.25,0.1]
        origin = "TopLeft"
        "#;
    let config: GridCfg = parse_config(toml.to_string(), "").unwrap();
    let grid = Grid::from_config(&config).unwrap();
    assert_eq!(
        grid.extent,
        Extent {
            minx: 2420000.0,
            miny: 1030000.0,
            maxx: 2900000.0,
            maxy: 1350000.0,
        }
    );
    assert_eq!(grid.origin, Origin::TopLeft);

    let extent = grid.tile_extent(10, 4, 17); // lake of Zurich
    assert_eq!(
        extent,
        Extent {
            minx: 2676000.,
            miny: 1222000.,
            maxx: 2701600.,
            maxy: 1247600.,
        }
    );
    //BBOX ZH: (2669255.48 1223902.28, 2716899.60125 1283304.23625)
    let extent = grid.tile_extent_xyz(10, 4, 17);
    assert_eq!(
        extent,
        Extent {
            minx: 2676000.,
            miny: 1119600.0,
            maxx: 2701600.,
            maxy: 1145200.0,
        }
    );
}
