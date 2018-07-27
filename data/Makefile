DBNAME = t_rex_tests

createdb:
	psql postgres -c "DROP DATABASE IF EXISTS $(DBNAME)"
	psql postgres -c "CREATE DATABASE $(DBNAME)"
	psql $(DBNAME) -c "CREATE EXTENSION postgis"
	psql $(DBNAME) -c "CREATE SCHEMA ne"
	psql $(DBNAME) -c "CREATE SCHEMA avch"
	psql $(DBNAME) -c "CREATE SCHEMA geostat"

loaddata:
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) -lco SCHEMA=ne natural_earth.gpkg
	psql $(DBNAME) -c "CREATE TABLE ne.rivers_lake_centerlines AS SELECT fid,scalerank,name,(ST_DUMP(wkb_geometry)).geom::geometry(LineString,3857) AS wkb_geometry FROM ne.ne_10m_rivers_lake_centerlines"
	psql $(DBNAME) -c "CREATE TABLE ne.admin_0_countries AS SELECT fid,name,iso_a3,(ST_DUMP(wkb_geometry)).geom::geometry(Polygon,3857) AS wkb_geometry FROM ne.ne_110m_admin_0_countries"
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) -lco SCHEMA=avch avch.gpkg
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) -a_srs EPSG:2056 -nlt PROMOTE_TO_MULTI -lco SCHEMA=geostat g1k18.shp

dump:
	pg_dump -Fc --no-owner -f t_rex_tests.dump $(DBNAME)

# Original creation of natural_earth.gpkg from Natural_Earth_quick_start 2.0

Natural_Earth_quick_start.zip:
	wget -O $@ http://naciscdn.org/naturalearth/packages/Natural_Earth_quick_start.zip

packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp: Natural_Earth_quick_start.zip
	unzip $<
	touch $@

natural_earth.gpkg: packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -select scalerank,name,pop_max -lco FID=fid -f GPKG $@ packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -append -select scalerank,name -nlt PROMOTE_TO_MULTI -lco FID=fid -f GPKG $@ packages/Natural_Earth_quick_start/10m_physical/ne_10m_rivers_lake_centerlines.shp
	OGR_ENABLE_PARTIAL_REPROJECTION=TRUE SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -append -nlt PROMOTE_TO_MULTI -lco FID=fid -f GPKG $@ -select name,iso_a3 packages/Natural_Earth_quick_start/110m_cultural/ne_110m_admin_0_countries.shp

natural_earth.sqlite: natural_earth.gpkg
	ogr2ogr -f SQLite $@ $<

# Original creation of avch.gpkg

avch.gpkg: ili2pg
	ogr2ogr -f GPKG avch.gpkg PG:dbname=avch boflaeche liegenschaft strassenstueck hausnummerpos

ili2pg:
	ili2pg --import --dbdatabase avch --dbusr $$USER --dbpwd xxx ch_254900.itf

# Swiss grid test data

# https://www.bfs.admin.ch/bfs/de/home/dienstleistungen/geostat/geodaten-bundesstatistik/administrative-grenzen/generalisierte-gemeindegrenzen.html
gd-b-00.03-875-gg18.zip:
	wget -O gd-b-00.03-875-gg18.zip https://www.bfs.admin.ch/bfsstatic/dam/assets/5247306/master

g1k18.shp: gd-b-00.03-875-gg18.zip
	unzip -j gd-b-00.03-875-gg18.zip ggg_2018-LV95/shp/g1k18.*
	touch $@
