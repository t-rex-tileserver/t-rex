use datasource::datasource::Datasource;
use postgres::{Connection, SslMode};
use postgres::rows::Row;
use core::feature::{Feature,FeatureAttr};
use core::geom::*;
use core::grid::Extent;
use core::layer::Layer;


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

struct FeatureRow<'a> {
    layer: &'a Layer,
    row: &'a Row<'a>,
    attrs: Vec<FeatureAttr>,  // temporary
}

impl<'a> Feature for FeatureRow<'a> {
    fn fid(&self) -> Option<u64> { None } //TODO
    fn attributes(&self) -> &Vec<FeatureAttr> { &self.attrs } //TODO
    fn geometry(&self) -> GeometryType {
        GeometryType::from_geom_field(
            &self.row,
            &self.layer.geometry_field(),
            &self.layer.geometry_type()
        )
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
}

impl Datasource for PostgisInput {
    fn retrieve_features<F>(&self, layer: &Layer, extent: &Extent, zoom: u16, mut read: F)
        where F : FnMut(&Feature) {
        let conn = Connection::connect(self.connection_url, SslMode::None).unwrap();
        let stmt = conn.prepare(&layer.query).unwrap();
        for row in &stmt.query(&[]).unwrap() {
            let feature = FeatureRow { layer: layer, row: &row, attrs: vec![] };
            read(&feature)
        }
    }
}


#[cfg(feature = "dbtest")]
#[test]
pub fn test_from_geom_fields() {
    let conn = Connection::connect("postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles", SslMode::None).unwrap();
    let stmt = conn.prepare("SELECT geometry FROM osm_place_point LIMIT 1").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        println!(">>>>>> {}", row.get::<_, Point>("geometry"));
        let geom = GeometryType::from_geom_field(&row, "geometry", "POINT");
        assert_eq!("Point(\n    SRID=3857;POINT(921771.0175818551 5981453.77061269)\n)", &*format!("{:#?}", geom));
    }
    /*
    let stmt = conn.prepare("SELECT geometry FROM osm_water_linestring LIMIT 2").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        println!(">>>>>> {}", row.get::<_, LineString>("geometry"));
    }*/
}

#[cfg(feature = "dbtest")]
#[test]
pub fn test_detect_layers() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    //"postgresql://pi@localhost/osm2vectortiles";
    let table_name = pg.detect_layers();
    assert_eq!(table_name, "osm_admin_linestring");
}

#[cfg(feature = "dbtest")]
#[test]
pub fn test_retrieve_features() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let layer = Layer {
        name: String::from("points"),
        query: String::from("SELECT geometry FROM osm_place_point LIMIT 1")
    };
    let extent = Extent {minx: 958826.08, miny: 5987771.04, maxx: 978393.96, maxy: 6007338.92};
    pg.retrieve_features(&layer, &extent, 10, |feat| {
        assert_eq!("Point(\n    SRID=3857;POINT(921771.0175818551 5981453.77061269)\n)", &*format!("{:#?}", feat.geometry()));
        assert_eq!(0, feat.attributes().len());
        assert_eq!(None, feat.fid());
    });
}
