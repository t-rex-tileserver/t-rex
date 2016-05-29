//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use datasource::DatasourceInput;
use postgres::{Connection, SslMode};
use postgres::rows::Row;
use postgres::types::Type;
use core::feature::{Feature,FeatureAttr,FeatureAttrValType};
use core::geom::*;
use core::grid::Extent;
use core::layer::Layer;
use config::Config;
use toml;


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
}

impl<'a> Feature for FeatureRow<'a> {
    fn fid(&self) -> Option<u64> {
        self.layer.fid_field.as_ref().and_then(|fid| {
            let val: i32 = self.row.get(fid as &str); //TODO: could be also i16 or i64
            Some(val as u64)
        })
    }
    fn attributes(&self) -> Vec<FeatureAttr> {
        let mut attrs = Vec::new();
        for (i,col) in self.row.columns().into_iter().enumerate() {
            if col.name() != self.layer.geometry_field.as_ref().unwrap_or(&"".to_string()) {
                let fattr = FeatureAttr {
                    key: col.name().to_string(),
                    value: match col.type_() {
                        // http://sfackler.github.io/rust-postgres/doc/v0.11.8/postgres/types/enum.Type.html
                        // http://sfackler.github.io/rust-postgres/doc/v0.11.8/postgres/types/trait.FromSql.html#types
                        &Type::Varchar | &Type::Text | &Type::CharArray
                            => FeatureAttrValType::String(self.row.get(i)),
                        &Type::Float4
                            => FeatureAttrValType::Float(self.row.get(i)),
                        &Type::Float8
                            => FeatureAttrValType::Double(self.row.get(i)),
                        &Type::Int2
                            => FeatureAttrValType::Int(self.row.get::<_, i16>(i) as i64),
                        &Type::Int4
                            => FeatureAttrValType::Int(self.row.get::<_, i32>(i) as i64),
                        &Type::Int8
                            => FeatureAttrValType::Int(self.row.get(i)),
                        &Type::Bool
                            => FeatureAttrValType::Bool(self.row.get(i)),
                        _ => FeatureAttrValType::Int(0)
                    }
                };
                attrs.push(fattr);
            }
        }
        attrs
    }
    fn geometry(&self) -> GeometryType {
        GeometryType::from_geom_field(
            &self.row,
            &self.layer.geometry_field.as_ref().unwrap(),
            &self.layer.geometry_type.as_ref().unwrap()
        )
    }
}

pub struct PostgisInput {
    pub connection_url: String
}

impl PostgisInput {
    pub fn detect_layers(&self) -> Vec<Layer> {
        let mut layers: Vec<Layer> = Vec::new();
        let conn = Connection::connect(&self.connection_url as &str, SslMode::None).unwrap();
        let stmt = conn.prepare("SELECT * FROM geometry_columns").unwrap();
        for row in &stmt.query(&[]).unwrap() {
            let table_name: String = row.get("f_table_name");
            let geometry_column: String = row.get("f_geometry_column");
            let srid: i32 = row.get("srid");
            let geomtype: String = row.get("type");
            let mut layer = Layer::new(&table_name);
            layer.table_name = Some(table_name.clone());
            layer.geometry_field = Some(geometry_column.clone());
            layer.geometry_type = Some(geomtype.clone());
            layers.push(layer);
        }
        layers
    }
    fn query(&self, layer: &Layer, zoom: u16) -> String {
        let mut sql = match layer.query.as_ref() {
            Some(q) => q.clone(),
            None => format!("SELECT {} FROM {}",
                layer.geometry_field.as_ref().unwrap(),
                layer.table_name.as_ref().unwrap())
        };
        sql.push_str(&format!(" WHERE ST_Intersects({},ST_MakeEnvelope($1,$2,$3,$4,3857))",
            layer.geometry_field.as_ref().unwrap()));
        if let Some(n) = layer.query_limit {
            sql.push_str(&format!(" LIMIT {}", n));
        }
        sql
    }
}

impl DatasourceInput for PostgisInput {
    fn retrieve_features<F>(&self, layer: &Layer, extent: &Extent, zoom: u16, mut read: F)
        where F : FnMut(&Feature) {
        let conn = Connection::connect(&self.connection_url as &str, SslMode::None).unwrap();
        let stmt = conn.prepare(&self.query(&layer, zoom)).unwrap();
        for row in &stmt.query(&[&extent.minx, &extent.miny, &extent.maxx, &extent.maxy]).unwrap() {
            let feature = FeatureRow { layer: layer, row: &row };
            read(&feature)
        }
    }
}

impl Config<PostgisInput> for PostgisInput {
    fn from_config(config: &toml::Value) -> Result<Self, String> {
        config.lookup("datasource.url")
            .ok_or("Missing configuration entry 'datasource.url'".to_string())
            .and_then(|val| val.as_str().ok_or("url entry is not a string".to_string()))
            .and_then(|url| Ok(PostgisInput { connection_url: url.to_string() }))
    }

    fn gen_config() -> String {
        let toml = r#"
[datasource]
type = "postgis"
# Connection specification (https://github.com/sfackler/rust-postgres#connecting)
url = "postgresql://user:pass@host:port/database"
"#;
        toml.to_string()
    }
    fn gen_runtime_config(&self) -> String {
        format!(r#"
[datasource]
type = "postgis"
url = "{}"
"#, self.connection_url)
    }
}

#[cfg(test)] use std::io::{self,Write};
#[cfg(test)] use std::env;

#[test]
pub fn test_from_geom_fields() {
    use postgis;
    use postgres;

    let conn: Connection = match env::var("DBCONN") {
        Result::Ok(val) => Connection::connect(&val as &str, SslMode::None),
        Result::Err(_) => { write!(&mut io::stdout(), "skipped ").unwrap(); return; }
    }.unwrap();
    let stmt = conn.prepare("SELECT wkb_geometry FROM ne_10m_populated_places LIMIT 1").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        let geom = row.get::<_, Point>("wkb_geometry");
        assert_eq!(&*format!("{:?}", geom),
            "SRID=3857;POINT(-6438719.622820721 -4093437.7144101723)");
        let geom = GeometryType::from_geom_field(&row, "wkb_geometry", "POINT");
        assert_eq!(&*format!("{:?}", geom),
            "Point(SRID=3857;POINT(-6438719.622820721 -4093437.7144101723))");
    }

    let stmt = conn.prepare("SELECT wkb_geometry FROM rivers_lake_centerlines WHERE ST_NPoints(wkb_geometry)<10 LIMIT 1").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        let geom = GeometryType::from_geom_field(&row, "wkb_geometry", "LINESTRING");
        assert_eq!(&*format!("{:?}", geom),
            "LineString(LineString { points: [SRID=3857;POINT(18672061.098933436 -5690573.725394946), SRID=3857;POINT(18671798.382036217 -5692123.11701991), SRID=3857;POINT(18671707.790002696 -5693530.713572942), SRID=3857;POINT(18671789.322832868 -5694822.281317252), SRID=3857;POINT(18672061.098933436 -5695997.770001522), SRID=3857;POINT(18670620.68560042 -5698245.837796968), SRID=3857;POINT(18668283.41113552 -5700403.997584983), SRID=3857;POINT(18666082.024720907 -5701179.511527114), SRID=3857;POINT(18665148.926775623 -5699253.775757339)] })");
    }
    /* row.get panics for multi-geometries: https://github.com/andelf/rust-postgis/issues/6
    let stmt = conn.prepare("SELECT wkb_geometry FROM ne_10m_rivers_lake_centerlines WHERE ST_NPoints(wkb_geometry)<10 LIMIT 1").unwrap();
    for row in &stmt.query(&[]).unwrap() {
        let geom = row.get::<_, postgis::MultiLineString<postgis::Point<EPSG_3857>>>("wkb_geometry");
        assert_eq!(&*format!("{:#?}", geom),
            "SRID=3857;MULTILINESTRING((5959308.21223679 7539958.36540974,5969998.07219252 7539958.36540974,5972498.41231776 7539118.00291568,5977308.84929784 7535385.96203562))");
    }*/

    let stmt = conn.prepare("SELECT wkb_geometry, ST_AsBinary(wkb_geometry) FROM rivers_lake_centerlines LIMIT 1").unwrap();
    let rows = &stmt.query(&[]).unwrap();
    assert_eq!(rows.columns()[0].name(), "wkb_geometry");
    assert_eq!(format!("{}", rows.columns()[0].type_()), "geometry");
    assert_eq!(rows.columns()[1].name(), "st_asbinary");
    assert_eq!(format!("{}", rows.columns()[1].type_()), "bytea");
}

#[test]
pub fn test_detect_layers() {
    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput {connection_url: val}),
        Result::Err(_) => { write!(&mut io::stdout(), "skipped ").unwrap(); return; }
    }.unwrap();
    let layers = pg.detect_layers();
    assert_eq!(layers[0].name, "ne_10m_populated_places");
}

#[test]
pub fn test_feature_query() {
    let pg = PostgisInput {connection_url: "postgresql://pi@localhost/osm2vectortiles".to_string()};
    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("osm_place_point"));
    layer.geometry_field = Some(String::from("geometry"));
    assert_eq!(pg.query(&layer, 10),
        "SELECT geometry FROM osm_place_point WHERE ST_Intersects(geometry,ST_MakeEnvelope($1,$2,$3,$4,3857))");

    layer.query_limit = Some(1);
    assert_eq!(pg.query(&layer, 10),
        "SELECT geometry FROM osm_place_point WHERE ST_Intersects(geometry,ST_MakeEnvelope($1,$2,$3,$4,3857)) LIMIT 1");

    layer.query = Some(String::from("SELECT geometry AS geom FROM osm_place_point"));
    assert_eq!(pg.query(&layer, 10),
        "SELECT geometry AS geom FROM osm_place_point WHERE ST_Intersects(geometry,ST_MakeEnvelope($1,$2,$3,$4,3857)) LIMIT 1");
}

#[test]
pub fn test_retrieve_features() {
    let pg: PostgisInput = match env::var("DBCONN") {
        Result::Ok(val) => Some(PostgisInput {connection_url: val}),
        Result::Err(_) => { write!(&mut io::stdout(), "skipped ").unwrap(); return; }
    }.unwrap();

    let mut layer = Layer::new("points");
    layer.table_name = Some(String::from("ne_10m_populated_places"));
    layer.geometry_field = Some(String::from("wkb_geometry"));
    layer.geometry_type = Some(String::from("POINT"));
    let extent = Extent { minx: 821850.9, miny: 5909499.5, maxx: 860986.7, maxy: 5948635.3 };

    let mut reccnt = 0;
    pg.retrieve_features(&layer, &extent, 10, |feat| {
        assert_eq!("Point(SRID=3857;POINT(831219.9062494118 5928485.165733484))", &*format!("{:?}", feat.geometry()));
        assert_eq!(0, feat.attributes().len());
        assert_eq!(None, feat.fid());
        reccnt += 1;
    });
    assert_eq!(1, reccnt);

    layer.query = Some(String::from("SELECT * FROM ne_10m_populated_places"));
    layer.fid_field = Some(String::from("fid"));
    pg.retrieve_features(&layer, &extent, 10, |feat| {
        assert_eq!("Point(SRID=3857;POINT(831219.9062494118 5928485.165733484))", &*format!("{:?}", feat.geometry()));
        assert_eq!(feat.attributes()[0].key, "fid");
        assert_eq!(feat.attributes()[1].key, "scalerank");
        assert_eq!(feat.attributes()[2].key, "name");
        assert_eq!(feat.attributes()[3].key, "pop_max");
        assert_eq!(feat.attributes()[0].value, FeatureAttrValType::Int(6478));
        assert_eq!(feat.attributes()[2].value, FeatureAttrValType::String("Bern".to_string()));
        assert_eq!(feat.fid(), Some(6478));
    });

}
