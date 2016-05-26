t-rex
=====

t-rex is a vector tile server specialized on publishing [MVT tiles](https://github.com/mapbox/vector-tile-spec/tree/master/2.1)
from a PostGIS database.

An extensible design allows future support for more data sources (e.g. OGR), custom tile
grids with other reference systems than Spherical Mercator and additional output formats like
JSON.

Usage
-----

    t_rex serve --dbconn postgresql://pi@localhost/osm2vectortiles

Tiles are then served at `http://localhost:6767/{layer}/{z}/{x}/{y}.pbf`

A list of all detected layers is available at [http://localhost:6767/](http://localhost:6767/)

Generate a configuration template:

    t_rex genconfig --dbconn postgresql://pi@localhost/osm2vectortiles

Run server with configuration file:

    t_rex serve --config osm2vectortiles.cfg


For developers
--------------

t-rex is written in [Rust](https://www.rust-lang.org/).

Build:

    cargo build

Run tests:

    cargo test

To run DB tests you have to set an environment variable with the [connection spec](https://github.com/sfackler/rust-postgres#connecting) first. Example:

     export DBCONN=postgresql://pi@localhost/natural_earth_vectors

Creating test database:

    # Set Postgresql environment variables when needed: PGHOST, PGPORT, PGUSER, PGPASSWORD
    cd src/test
    make

Run server:

    cargo run -- serve --dbconn postgresql://pi@%2Frun%2Fpostgresql/natural_earth_vectors


License
-------

t-rex is released under the MIT License.
