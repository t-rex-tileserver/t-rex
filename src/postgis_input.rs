use postgres::{Connection, SslMode};
//https://github.com/andelf/rust-postgis
//use postgis::{Point, LineString, WGS84};

#[test]
pub fn load() {
    let url = "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles";
    //let url = "postgresql://pi@localhost/osm2vectortiles";
    let conn = Connection::connect(url, SslMode::None).unwrap();
    let stmt = conn.prepare("SELECT * FROM geometry_columns").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        let table_name: String = row.get("f_table_name");
        println!("{}", table_name);
    }
    let rows = conn.query("SELECT * FROM geometry_columns LIMIT 1", &[]).unwrap();
    let table_name: String = rows.get(0).get("f_table_name");
    assert_eq!(table_name, "osm_admin_linestring");
}
