use postgres::{Connection, SslMode};
use postgres::rows::Row;
use core::geom::*;
use core::grid::Extent;


impl GeometryType {
    fn from_geom_field(row: &Row, idx: &str, type_name: &str) -> GeometryType {
        match type_name {
            "POINT"              => GeometryType::Point(row.get::<_, Point>(idx)),
            "LINESTRING"         => GeometryType::LineString(row.get::<_, LineString>(idx)),
            "POLYGON"            => GeometryType::Polygon(row.get::<_, Polygon>(idx)),
            "MULTIPOINT"         => GeometryType::MultiPoint(row.get::<_, MultiPoint>(idx)),
            "MULTILINESTRING"    => GeometryType::MultiLineString(row.get::<_, MultiLineString>(idx)),
            "MULTIPOLYGON"       => GeometryType::MultiPolygon(row.get::<_, MultiPolygon>(idx)),
            "GEOMETRYCOLLECTION" => GeometryType::GeometryCollection(row.get::<_, GeometryCollection>(idx)),
            _                    => panic!("Unknown geometry type")
        }
    }
}

pub struct PostgisInput {
    pub connection_url: &'static str
}

impl PostgisInput {
    pub fn detect_layers(&self) -> String {
        let conn = Connection::connect(self.connection_url, SslMode::None).unwrap();
        let stmt = conn.prepare("SELECT * FROM geometry_columns").unwrap();
        for row in &stmt.query(&[]).unwrap() {
            let table_name: String = row.get("f_table_name");
            println!("{}", table_name);
        }
        let rows = conn.query("SELECT * FROM geometry_columns LIMIT 1", &[]).unwrap();
        let table_name: String = rows.get(0).get("f_table_name");
        table_name
    }
    pub fn select_geometries(&self) {
        let conn = Connection::connect(self.connection_url, SslMode::None).unwrap();
        let stmt = conn.prepare("SELECT geometry FROM osm_place_point LIMIT 2").unwrap();
        for row in &stmt.query(&[]).unwrap() {
            let geom = GeometryType::from_geom_field(&row, "geometry", "POINT");
            println!(">>>>>> {}", row.get::<_, Point>("geometry"));
        }
        let stmt = conn.prepare("SELECT geometry FROM osm_water_linestring LIMIT 2").unwrap();
        for row in &stmt.query(&[]).unwrap() {
            println!(">>>>>> {}", row.get::<_, LineString>("geometry"));
        }
    }
    pub fn get_features(&self, layer: &str, extent: &Extent) -> Point {
        Point::new(960000.0, 6002729.0)
    }
}

#[cfg(feature = "dbtest")]
#[test]
pub fn test_geometry_types() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    pg.select_geometries();
}

#[cfg(feature = "dbtest")]
#[test]
pub fn test_detect_layers() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    //"postgresql://pi@localhost/osm2vectortiles";
    let table_name = pg.detect_layers();
    assert_eq!(table_name, "osm_admin_linestring");
}
