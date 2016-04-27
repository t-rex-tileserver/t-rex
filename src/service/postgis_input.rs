use postgres::{Connection, SslMode};

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
}

#[test]
pub fn test_detect_layers() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    //"postgresql://pi@localhost/osm2vectortiles";
    let table_name = pg.detect_layers();
    assert_eq!(table_name, "osm_admin_linestring");
}
