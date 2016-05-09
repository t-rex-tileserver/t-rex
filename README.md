t-rex
=====

t-rex is a vector tile server specialized on publishing [MVT tiles](https://github.com/mapbox/vector-tile-spec/tree/master/2.1)
from a PostGIS database.

An extensible design allows future support for additional data sources (e.g. OGR), custom tile
grids with other reference systems than Spherical Mercator and addional outputs like
JSON.

Usage
-----

    t_rex serve --dbconnection=postgresql://pi@localhost/osm2vectortiles

Tiles are then served at http://localhost:6767/<topic>/z/x/y.pbf

Additional commands (not implemented yet):

    t_rex genconfig --dbconnection=postgresql://pi@localhost/osm2vectortiles

    t_rex seed --config=osm2vectortiles.cfg


For developers
--------------

t-rex is written in [Rust](https://www.rust-lang.org/).

Build:

    cargo build

Run tests:

    cargo test

With DB tests:

    cargo test --features=dbtest


License
-------

t-rex is released under the MIT License.
