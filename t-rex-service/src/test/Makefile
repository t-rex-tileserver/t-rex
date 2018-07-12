DBNAME = natural_earth_vectors

createdb:
	psql postgres -c "DROP DATABASE IF EXISTS $(DBNAME)"
	psql postgres -c "CREATE DATABASE $(DBNAME)"
	psql $(DBNAME) -c "CREATE EXTENSION postgis"

loadfromgpkg:
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) ../../../t-rex-gdal/natural_earth.gpkg
	psql $(DBNAME) -c "CREATE TABLE rivers_lake_centerlines AS SELECT fid,scalerank,name,(ST_DUMP(wkb_geometry)).geom::geometry(LineString,3857) AS wkb_geometry FROM ne_10m_rivers_lake_centerlines"
	psql $(DBNAME) -c "CREATE TABLE admin_0_countries AS SELECT fid,name,iso_a3,(ST_DUMP(wkb_geometry)).geom::geometry(Polygon,3857) AS wkb_geometry FROM ne_110m_admin_0_countries"

loadwgdal1:
	@# Support GDAL 1.10 on Travis
	ogr2ogr -f PostgreSQL PG:dbname=$(DBNAME) ../../../t-rex-gdal/natural_earth.sqlite
	psql $(DBNAME) -c "CREATE TABLE rivers_lake_centerlines AS SELECT ogc_fid AS fid,scalerank,name,(ST_DUMP(wkb_geometry)).geom::geometry(LineString,3857) AS wkb_geometry FROM ne_10m_rivers_lake_centerlines"
	psql $(DBNAME) -c "CREATE TABLE admin_0_countries AS SELECT ogc_fid AS fid,name,iso_a3,(ST_DUMP(wkb_geometry)).geom::geometry(Polygon,3857) AS wkb_geometry FROM ne_110m_admin_0_countries"

dump:
	pg_dump -Fc --no-owner -f ne_t_rex_test.dump $(DBNAME)

# Original creation of natural_earth.gpkg from Natural_Earth_quick_start 2.0

Natural_Earth_quick_start.zip:
	wget -O $@ http://naciscdn.org/naturalearth/packages/Natural_Earth_quick_start.zip

packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp: Natural_Earth_quick_start.zip
	unzip $<
	touch $@

../../../t-rex-gdal/natural_earth.gpkg: packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -select scalerank,name,pop_max -lco FID=fid -f GPKG $@ packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -append -select scalerank,name -nlt PROMOTE_TO_MULTI -lco FID=fid -f GPKG $@ packages/Natural_Earth_quick_start/10m_physical/ne_10m_rivers_lake_centerlines.shp
	OGR_ENABLE_PARTIAL_REPROJECTION=TRUE SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -append -nlt PROMOTE_TO_MULTI -lco FID=fid -f GPKG $@ -select name,iso_a3 packages/Natural_Earth_quick_start/110m_cultural/ne_110m_admin_0_countries.shp

../../../t-rex-gdal/natural_earth.sqlite: ../../../t-rex-gdal/natural_earth.gpkg
	ogr2ogr -f SQLite $@ $<

# Swiss grid test data
# https://www.bfs.admin.ch/bfsstatic/dam/assets/453578/master
geostat:
	psql postgres -c "DROP DATABASE IF EXISTS geostat"
	psql postgres -c "CREATE DATABASE geostat"
	psql geostat -c "CREATE EXTENSION postgis"
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:2056 -nlt PROMOTE_TO_MULTI -f PostgreSQL PG:dbname=geostat geostat_ggg_16/shp/g1k16.shp
	# ggg_2017-LV95/shp
	#for i in *.shp; do ogr2ogr -a_srs EPSG:2056 -nlt PROMOTE_TO_MULTI -f PostgreSQL PG:dbname=geostat $i; done
