use postgres::{Connection, SslMode};
use postgis as geom;
use core::grid::{Extent,EPSG_3857};

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
            println!(">>>>>> {}", row.get::<_, geom::Point<EPSG_3857>>("geometry"));
        }
        let stmt = conn.prepare("SELECT geometry FROM osm_water_linestring LIMIT 2").unwrap();
        for row in &stmt.query(&[]).unwrap() {
            println!(">>>>>> {}", row.get::<_, geom::LineString<geom::Point<EPSG_3857>>>("geometry"));
        }
    }
    pub fn get_features(&self, layer: &str, extent: &Extent) -> geom::Point<EPSG_3857> {
        geom::Point::<EPSG_3857>::new(960000.0, 6002729.0)
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
