tile-grid
=========

[![Crates.io](https://img.shields.io/crates/v/tile-grid.svg?maxAge=2592000)](https://crates.io/crates/tile-grid)
[![Documentation](https://docs.rs/tile-grid/badge.svg)](https://docs.rs/tile-grid/)

tile-grid is a library for map tile grid calculations.

Included standard grids are Web Mercator and WGS 84.

Usage
-----

### Predefined grids

```rust
use tile_grid::{Extent, Grid};

let grid = Grid::wgs84();
assert_eq!(
    grid.tile_extent(0, 0, 0),
    Extent {
        minx: -180.0,
        miny: -90.0,
        maxx: 0.0,
        maxy: 90.0,
    }
);
```

### Grid iterators

```rust
use tile_grid::{Grid, GridIterator};

let grid = Grid::web_mercator();
let tile_limits = grid.tile_limits(grid.extent.clone(), 0);
let griditer = GridIterator::new(0, 2, tile_limits);
for (z, x, y) in griditer {
    println!("Tile {}/{}/{}", z, x, y);
}
```

### Custom grids

```rust
use tile_grid::{Extent, Grid, Unit, Origin};

let grid = Grid::new(
    256,
    256,
    Extent {
        minx: 2420000.0,
        miny: 1030000.0,
        maxx: 2900000.0,
        maxy: 1350000.0,
    },
    2056,
    Unit::Meters,
    vec![
        4000.0, 3750.0, 3500.0, 3250.0, 3000.0, 2750.0, 2500.0, 2250.0, 2000.0, 1750.0, 1500.0,
        1250.0, 1000.0, 750.0, 650.0, 500.0, 250.0, 100.0, 50.0, 20.0, 10.0, 5.0, 2.5, 2.0,
        1.5, 1.0, 0.5,
    ],
    Origin::TopLeft,
);
assert_eq!(
    grid.tile_extent(0, 0, 15),
    Extent {
        minx: 2420000.0,
        miny: 1222000.0,
        maxx: 2548000.0,
        maxy: 1350000.0,
    }
);
```

Credits
-------

* [MapCache](https://mapserver.org/mapcache/) by Thomas Bonfort
* [Mercantile](https://github.com/mapbox/mercantile) by Sean C. Gillies


License
-------

tile-grid is released under the MIT License.
