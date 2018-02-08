<a name="0.8.2"></a>
## 0.8.2 (UNRELEASED)

#### Features

* Use tileset extent when generating cache (Thanks @lnicola!)

<a name="0.8.1"></a>
## 0.8.1 (2017-10-05)

#### Features

* Read layers from QGIS project file

#### Bug Fixes

* Fix extent calculation for reprojected OGR geometries

<a name="0.8.0"></a>
## 0.8.0 (2017-09-26)

#### Features

* Support for GDAL/OGR datasources (up to 84 drivers, see http://gdal.org/)
* Multiple datasources in tilesets
* Configurable layer tile size
* Options `bind` and `port` for `serve` command
* New package formats: Deb package for Ubuntu, MSI for Windows and Docker image
  for all platforms

#### Breaking changes

* Changed configuration format of datasources

  See [Upgrading](http://t-rex.tileserver.ch/doc/setup/#0-7-x-0-8-0) for conversion help.
* User defined grid now in [grid.user] block
* Changed user defined grid units to lower case (m, dd, ft)

<a name="0.7.8"></a>
## 0.7.8 (2017-08-05)

#### Features

* Support for PostgreSQL SSL connections
* Serve fonts in pbf format

<a name="0.7.7"></a>
## 0.7.7 (2017-07-14)

#### Bug Fixes

* Fix queries with `!zoom!` variable
* Fix queries for layers with minzoom > 0

#### Breaking changes

* Use OGC SLD pixel size for `scale_denominator` calculation (like Mapnik)

<a name="0.7.6"></a>
## 0.7.6 (2017-07-10)

#### Features

* Use tileset extent from configuration as default for cache generation
* Serve fontstacks.json (used by Maputnik et al.)

#### Bug Fixes

* Fix queries at maxzoom levels

#### Breaking changes

* Extent parameter of generate command is now in WGS84 instead of grid SRS

<a name="0.7.5"></a>
## 0.7.5 (2017-06-26)

#### Bug Fixes

* Fix tileset extent detection of empty tables

<a name="0.7.4"></a>
## 0.7.4 (2017-06-25)

#### Features

* Tileset extent detection and configuration
* New seeding option `overwrite` (Thanks @kiambogo!)
* cache_control_max_age setting (with new default 0)
* Write :tileset.json and and :tileset.style.json when generating cache

<a name="0.7.3"></a>
## 0.7.3 (2017-06-15)

#### Features

* Update built-in Mapbox GL viewer to 0.38.0
* Open backend URL in browser when starting server
* Add support for the environment variable TREX_DATASOURCE_URL to
  override the datasource.url config field (Thanks @kiambogo!)
* Serve favicon
* Emit info message when features are limited by `query_limit`

#### Bug Fixes

* Turn off HTTP keep alive to avoid missing tiles in browser
* Fix WGS84 grid definition (Thanks @Wykks!)

#### Breaking changes

* Use `buffer_size` instead of `buffer-size` in config

<a name="0.7.2"></a>
## 0.7.2 (2017-06-08)

#### Features

* Change file cache scheme from TMS to XYZ
* Extend Web Mercator grid to level 22
* Limit features per tile to 1000 by default

#### Bug Fixes

* Fix TileJSON compatibility
* Fix generation of zoom levels greater than maximal grid zoom level

<a name="0.7.1"></a>
## 0.7.1 (2017-04-01)

#### Features

* Embedded [Maputnik](https://github.com/maputnik/editor) style editor
* Service Info page with viewer code snippets 

#### Bug Fixes

* Fix `generate` command with `extent` option

<a name="0.7.0"></a>
## 0.7.0 (2017-03-12)

#### Features

* Inline Mapbox GL (TOML) styles

#### Bug Fixes

* Extent and zoom calculation fixes (Thanks @rory and @joostvenema!)

<a name="0.6.1"></a>
## 0.6.1 (2016-11-22)

#### Features

* Support for PostgreSQL Unix socket connections
* Mac OS X build

#### Bug Fixes

* Fix integer overflows in grid calculations and MVT encoding

<a name="0.6.0"></a>
## 0.6.0 (2016-11-07)

#### Features

* New viewer with inlined OpenLayers and Mapbox GL libs
* User defined grids

![user_grid](doc/lv95grid.jpg)

#### Bug Fixes

* Fix clipping and simplification with reprojected geometries

<a name="0.5.0"></a>
## 0.5.0 (2016-10-25)

#### Features

* Experimental Mapbox GL Style Json output

#### Bug Fixes

* Support for Multi-Geometries (Multipoint, Multiline, Multipolygon)

<a name="v0.4.0"></a>
## v0.4.0 (2016-09-11)

#### Features

* Tile cache generation command
* Improved polygon simplification

![t_rex_generate](doc/t_rex_generate.gif)

<a name="v0.3.1"></a>
## v0.3.1 (2016-09-06)

#### Bug Fixes

* Support for database column names which have a colon 
* TileJSON center value format fixed


<a name="v0.3.0"></a>
## v0.3.0 (2016-09-05)

#### Features

* Simplify option
* Experimental clipping support


<a name="0.2.0"></a>
## 0.2.0 (2016-08-30)

#### Features

* Automatic column type conversion
* Transform geometries to grid SRS
* Pre-build SQL queries


<a name="0.1.0"></a>
## 0.1.0 (2016-08-17)

First Release
