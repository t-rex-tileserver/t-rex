all: natural_earth_vectors

Natural_Earth_quick_start.zip:
	wget -O $@ http://naciscdn.org/naturalearth/packages/Natural_Earth_quick_start.zip

packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp: Natural_Earth_quick_start.zip
	unzip $<
	touch $@

natural_earth_vectors: packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	psql postgres -c "DROP DATABASE IF EXISTS natural_earth_vectors"
	psql postgres -c "CREATE DATABASE natural_earth_vectors"
	psql natural_earth_vectors -c "CREATE EXTENSION postgis"
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -select scalerank,name,pop_max -lco FID=fid -f PostgreSQL PG:dbname=natural_earth_vectors packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -select scalerank,name -nlt PROMOTE_TO_MULTI -lco FID=fid -f PostgreSQL PG:dbname=natural_earth_vectors packages/Natural_Earth_quick_start/10m_physical/ne_10m_rivers_lake_centerlines.shp
	psql natural_earth_vectors -c "CREATE TABLE rivers_lake_centerlines AS SELECT fid,scalerank,name,(ST_DUMP(wkb_geometry)).geom::geometry(LineString,3857) AS wkb_geometry FROM ne_10m_rivers_lake_centerlines"
	OGR_ENABLE_PARTIAL_REPROJECTION=TRUE SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:3857 -nlt PROMOTE_TO_MULTI -lco FID=fid -f PostgreSQL PG:dbname=natural_earth_vectors -select name,iso_a3 packages/Natural_Earth_quick_start/110m_cultural/ne_110m_admin_0_countries.shp
	psql natural_earth_vectors -c "CREATE TABLE admin_0_countries AS SELECT fid,name,iso_a3,(ST_DUMP(wkb_geometry)).geom::geometry(Polygon,3857) AS wkb_geometry FROM ne_110m_admin_0_countries"
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -select scalerank,name,pop_max -lco FID=fid -nln ne_10m_populated_places_wgs84 -f PostgreSQL PG:dbname=natural_earth_vectors packages/Natural_Earth_quick_start/10m_cultural/ne_10m_populated_places.shp

# Swiss grid test data
# https://www.bfs.admin.ch/bfsstatic/dam/assets/453578/master
geostat:
	psql natural_earth_vectors -c "CREATE schema geostat"
	SHAPE_ENCODING="ISO-8859-1" ogr2ogr -t_srs EPSG:2056 -nlt PROMOTE_TO_MULTI -lco SCHEMA=geostat -f PostgreSQL PG:dbname=natural_earth_vectors geostat_ggg_16/shp/g1k16.shp
