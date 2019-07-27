tile-grid
=========

[![Crates.io](https://img.shields.io/crates/v/tile-grid.svg?maxAge=2592000)](https://crates.io/crates/tile-grid)
[![Documentation](https://docs.rs/tile-grid/badge.svg)](https://docs.rs/tile-grid/)

tile-grid is a library for map tile grid calculations.

Included standard grids are Web Mercator and WGS 84.

Usage
-----

```Rust
let grid = Grid::web_mercator();
let tile_limits = grid.tile_limits(grid.extent.clone(), 0);
let griditer = GridIterator::new(0, 2, tile_limits);
for (z, x, y) in griditer {
    println!("Tile {}/{}/{}", z, x, y);
}
```

Credits
-------

* [MapCache](https://mapserver.org/mapcache/) by Thomas Bonfort
* [Mercantile](https://github.com/mapbox/mercantile) by Sean C. Gillies


License
-------

tile-grid is released under the MIT License.
