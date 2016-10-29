t-rex
=====

[![Travis build Status](https://travis-ci.org/pka/t-rex.svg?branch=master)](https://travis-ci.org/pka/t-rex) [![Appveyor build status](https://ci.appveyor.com/api/projects/status/o60e9bu97i49lxyf?svg=true)](https://ci.appveyor.com/project/pka/t-rex)


t-rex is a vector tile server specialized on publishing [MVT tiles](https://github.com/mapbox/vector-tile-spec/tree/master/2.1)
from a PostGIS database.


Features
--------

* Auto-detection of layers in database
* Built-in viewers for data display and inspection
* Tile generation command with simple parallelization
* Automatic reprojection to grid CRS
* Support for custom tile grids

### Presentations

* Workshop "Vector Tiles", GEOSummit Bern 7.6.16: [slides](doc/t-rex_vector_tile_server.pdf)


Usage
-----

    t_rex serve --dbconn postgresql://user:pass@localhost/osm2vectortiles

Tiles are then served at `http://localhost:6767/{layer}/{z}/{x}/{y}.pbf`

A list of all detected layers is available at [http://localhost:6767/](http://localhost:6767/)

Use a tile cache:

    t_rex serve --dbconn postgresql://user:pass@localhost/osm2vectortiles --cache /tmp/mvtcache

Generate a configuration template:

    t_rex genconfig --dbconn postgresql://user:pass@localhost/osm2vectortiles

Run server with configuration file:

    t_rex serve --config osm2vectortiles.cfg

Generate tiles for cache:

    t_rex generate --config osm2vectortiles.cfg


Configuration
-------------

Services can be configured in a text file with [TOML](https://github.com/toml-lang/toml) syntax.

A good starting point is the template generated with the `genconfig` command.

Configuration file example:

```toml
[service.mvt]
viewer = true

[datasource]
type = "postgis"
url = "postgresql://user:pass@localhost/natural_earth_vectors"

[grid]
predefined = "web_mercator"

[[tileset]]
name = "osm"

[[tileset.layer]]
name = "points"
# Select all attributes of table:
table_name = "ne_10m_populated_places"
geometry_field = "wkb_geometry"
geometry_type = "POINT"
fid_field = "id"

[[tileset.layer]]
name = "buildings"
geometry_field = "geometry"
geometry_type = "POLYGON"
fid_field = "osm_id"
# Clip polygons with a buffer
buffer-size = 10
simplify = true
  # Queries for different zoom levels:
  [[tileset.layer.query]]
  sql = """
    SELECT name, type, 0 as osm_id, ST_Union(geometry) AS geometry
    FROM osm_buildings_gen0
    WHERE geometry && !bbox!
    GROUP BY name, type
    ORDER BY sum(area) DESC"""
  [[tileset.layer.query]]
  minzoom = 17
  maxzoom = 22
  sql = """
    SELECT name, type, osm_id, geometry
    FROM osm_buildings
    WHERE geometry && !bbox!
    ORDER BY area DESC"""

[cache.file]
base = "/var/cache/mvtcache"

[webserver]
bind = "0.0.0.0"
port = 8080
threads = 4
```

### Layer configuration

Custom queries can be configured as PostGIS SQL queries.

The following variables are replaced at runtime:

* `!bbox!`: Bounding box of tile
* `!zoom!`: Zoom level of tile request
* `!scale_denominator!`: Map scale of tile request
* `!pixel_width!`: Width of pixel in grid units

If an `fid_field` is declared, this field is used as the feature ID.

### Custom tile grids

t-rex has two built-in grids, `web_mercator` and `wgs84`. Here's an example showing how to define your own grid:

```toml
[grid]
width = 256
height = 256
extent = { minx = 2420000.0, miny = 1030000.0, maxx = 2900000.0, maxy = 1350000.0 }
srid = 2056
units = "M"
resolutions = [4000.0,3750.0,3500.0,3250.0,3000.0,2750.0,2500.0,2250.0,2000.0,1750.0,1500.0,1250.0,1000.0,750.0,650.0,500.0,250.0,100.0,50.0,20.0,10.0,5.0,2.5,2.0,1.5,1.0,0.5]
origin = "TopLeft"
```


Installation
------------

Pre-built binaries are available for 64 bit Linux and Windows. Download your binary from [github.com/pka/t-rex/releases](https://github.com/pka/t-rex/releases) and unpack it.

`t_rex` is an executable with very few dependencies, essentially `libgcc_s.so.1` on Linux and `msvcr120.dll` on Windows. If `msvcr120.dll` is missing, install `vcredist_x64.exe` from [here](https://www.microsoft.com/download/details.aspx?id=40784).


MBTiles creation
----------------

To create MBTiles files with vector tiles from a local cache you can use [MBUtil](https://github.com/mapbox/mbutil).

Example:

    mb-util --image_format=pbf /tmp/mvtcache/streets streets.mbtiles


For developers
--------------

t-rex is written in [Rust](https://www.rust-lang.org/).

Build:

    cargo build

Run tests:

    cargo test

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
    cd src/test
    make

Run the tests with

    cargo test -- --ignored


License
-------

t-rex is released under the MIT License.
