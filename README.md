t-rex
=====

[![Travis build Status](https://travis-ci.org/t-rex-tileserver/t-rex.svg?branch=master)](https://travis-ci.org/t-rex-tileserver/t-rex) [![Appveyor build status](https://ci.appveyor.com/api/projects/status/o60e9bu97i49lxyf?svg=true)](https://ci.appveyor.com/project/pka/t-rex) [![Language (Rust)](https://img.shields.io/badge/powered_by-Rust-blue.svg)](http://www.rust-lang.org/)


t-rex is a vector tile server specialized on publishing [MVT tiles](https://github.com/mapbox/vector-tile-spec/tree/master/2.1)
from your own data.


Features
--------

* Support for PostGIS databases and GDAL vector formats
* Auto-detection of layers in data source
* Built-in viewers for data display and inspection
* Tile generation command with simple parallelization
* Automatic reprojection to grid CRS
* Support for custom tile grids
* Embedded styles


### Presentations

* T-rex, a vector tile server for your own data ([FOSS4G 2017](http://2017.foss4g.org/)): [slides](https://t-rex.tileserver.ch/Vector-tiles-and-QGIS.pdf)
* Vector Tiles - Introduction & Usage with QGIS (User meeting Bern 21.6.17): [slides](https://t-rex.tileserver.ch/Vector-tiles-and-QGIS.pdf)
* Von WMS zu WMTS zu Vektor-Tiles ([FOSSGIS 2017](https://www.fossgis-konferenz.de/2017/programm/event.php?id=5233)): [Video](https://av.tib.eu/media/30549)
* Workshop "Vector Tiles" (GEOSummit Bern 7.6.16): [slides](https://t-rex.tileserver.ch/t-rex_vector_tile_server.pdf)


Usage
-----

* [Setup](https://t-rex.tileserver.ch/doc/setup/)
* [Serving vector tiles](https://t-rex.tileserver.ch/doc/serve/)
* [Generating vector tiles](https://t-rex.tileserver.ch/doc/generate/)
* [Configuration](https://t-rex.tileserver.ch/doc/configuration/)


Quick tour
----------

    t_rex serve --dbconn postgresql://user:pass@localhost/osm2vectortiles

Tiles are then served at `http://localhost:6767/{layer}/{z}/{x}/{y}.pbf`

A list of all detected layers is available at [http://localhost:6767/](http://localhost:6767/)

Use a tile cache:

    t_rex serve --dbconn postgresql://user:pass@localhost/osm2vectortiles --cache /tmp/mvtcache

Generate a configuration template:

    t_rex genconfig --dbconn postgresql://user:pass@localhost/osm2vectortiles | tee osm2vectortiles.toml

Run server with configuration file:

    t_rex serve --config osm2vectortiles.toml

Generate tiles for cache:

    t_rex generate --config osm2vectortiles.toml


For developers
--------------

t-rex is written in [Rust](https://www.rust-lang.org/). Minimal required rustc version is 1.26.

Build:

    cargo build

Run tests:

    cargo test --all

Run server:

    cargo run -- serve --dbconn postgresql://pi@%2Frun%2Fpostgresql/natural_earth_vectors

Set log level:

    RUST_LOG=debug  # error, warn, info, debug, trace

Decode a vector tile:

    curl --silent http://127.0.0.1:6767/ne_10m_populated_places/5/31/17.pbf | gunzip -d | protoc --decode=vector_tile.Tile src/mvt/vector_tile.proto


### Database tests

Unit tests which need a PostgreSQL connection are ignored by default.

To run the database tests, declare the [connection](https://github.com/sfackler/rust-postgres#connecting) in an
environment variable `DBCONN`. Example:

    export DBCONN=postgresql://user:pass@localhost/natural_earth_vectors

Creating test database:

    # Set Postgresql environment variables when needed: PGHOST, PGPORT, PGUSER, PGPASSWORD
    cd t-rex-service/src/test
    make

Run the tests with

    cargo test --all -- --ignored


Roadmap
-------

[See Github board](https://github.com/t-rex-tileserver/t-rex/projects/1)


License
-------

t-rex is released under the MIT License.
