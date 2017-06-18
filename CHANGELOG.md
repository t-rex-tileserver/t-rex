<a name="0.7.4"></a>
## 0.7.4 (UNRELEASED)

#### Features

* New seeding option `overwrite` (Thanks @kiambogo!)

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
