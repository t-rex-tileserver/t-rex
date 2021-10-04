t-rex
=====

[![CI build status](https://github.com/t-rex-tileserver/t-rex/workflows/CI/badge.svg)](https://github.com/t-rex-tileserver/t-rex/actions)
[![Language (Rust)](https://img.shields.io/badge/powered_by-Rust-blue.svg)](http://www.rust-lang.org/)
[![Discord Chat](https://img.shields.io/discord/598002550221963289.svg)](https://discord.gg/Fp2aape)
[![docker](https://img.shields.io/docker/v/sourcepole/t-rex?label=Docker%20image&sort=semver)](https://hub.docker.com/r/sourcepole/t-rex)


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


### Presentations

* T-rex, a vector tile server for your own data ([FOSS4G 2017](http://2017.foss4g.org/)): [slides](https://t-rex.tileserver.ch/Vector-tiles-and-QGIS.pdf)
* Vector Tiles - Introduction & Usage with QGIS (User meeting Bern 21.6.17): [slides](https://t-rex.tileserver.ch/Vector-tiles-and-QGIS.pdf)
* Von WMS zu WMTS zu Vektor-Tiles ([FOSSGIS 2017](https://www.fossgis-konferenz.de/2017/programm/event.php?id=5233)): [Video](https://av.tib.eu/media/30549)
* Workshop "Vector Tiles" (GEOSummit Bern 7.6.16): [slides](https://t-rex.tileserver.ch/t-rex_vector_tile_server.pdf)


### Examples

* [AdV Smart Mapping](https://adv-smart.de/applications_en.html)
* Swiss Ornithological Institute, [Birds of Switzerland](https://www.vogelwarte.ch/en/birds/birds-of-switzerland/)


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

Increase log level:

    t_rex serve --loglevel debug --dbconn postgresql://user:pass@localhost/osm2vectortiles



For developers
--------------

t-rex is written in [Rust](https://www.rust-lang.org/). Minimal required rustc version is 1.45.


### Software Requirements

Ubuntu 20.04 (Focal Fossa):

    sudo apt install cargo libssl-dev libgdal-dev

### Build and run

Build:

    cargo build

Run tests:

    cargo test --all

Run server with DB [connection](https://github.com/sfackler/rust-postgres/tree/postgres-v0.15.2#connecting):

    cargo run -- serve --dbconn postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests

Decode a vector tile:

    curl --silent http://127.0.0.1:6767/ne_10m_populated_places/5/31/17.pbf | protoc --decode=vector_tile.Tile t-rex-core/src/mvt/vector_tile.proto


### Database tests

Unit tests which need a PostgreSQL connection are ignored by default.

Start Test DB:

    docker run -p 127.0.0.1:5439:5432 -d --name trextestdb --rm sourcepole/trextestdb

To run the database tests, declare the connection in an environment variable `DBCONN`:

    export DBCONN=postgresql://t_rex:t_rex@127.0.0.1:5439/t_rex_tests

Run the tests with

    cargo test --all -- --ignored

Creating test database locally:

    # Set Postgresql environment variables when needed: PGHOST, PGPORT, PGUSER, PGPASSWORD
    cd data
    make createdb loaddata

### S3 tests

Unit tests which need a S3 connection are skipped by default.

Install [MinIO Client](https://github.com/minio/mc).

Start Test S3

    docker run -d --rm -p 9000:9000 -e MINIO_REGION_NAME=my-region -e MINIO_ACCESS_KEY=miniostorage -e MINIO_SECRET_KEY=miniostorage minio/minio server /data && sleep 5 && mc config host add local-docker http://localhost:9000 miniostorage miniostorage && mc mb local-docker/trex && mc policy set download local-docker/trex 

To run the S3 tests, declare that there is a S3 available in an environment vaiable `S3TEST`:

    export S3TEST=true

Run the tests with

    cargo test --all -- --ignored

License
-------

t-rex is released under the MIT License.
