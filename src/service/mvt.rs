use datasource::datasource::Datasource;
use datasource::postgis::PostgisInput;
use core::grid::{Extent,Grid};
use core::layer::Layer;
use mvt::tile::Tile;
use mvt::vector_tile;
use mvt::geom_to_proto::EncodableGeom;


pub struct Topic {
    pub name: String,
    pub layers: Vec<Layer>
}

/// Mapbox Vector Tile Service
pub struct MvtService {
    input: PostgisInput,
    grid: Grid,
    topics: Vec<Topic>
}

impl MvtService {
    fn get_topic(&self, name: &str) -> &Topic {
        self.topics.iter().find(|t| t.name == name).unwrap()
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let extent = self.grid.tile_extent_xyz(xtile, ytile, zoom);
        let mut tile = Tile::new(&extent, 4096);
        let topic = self.get_topic(topic);
        for layer in topic.layers.iter() {
            let mut mvt_layer = tile.new_layer(layer);
            self.input.retrieve_features(&layer, &extent, zoom, |feat| {
                tile.add_feature(&mut mvt_layer, feat);
            });
            tile.add_layer(mvt_layer);
        }
        tile.mvt_tile
    }
}

#[cfg(feature = "dbtest")]
#[test]
pub fn test_tile_query() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let grid = Grid::web_mercator();
    let layers = vec![Layer {
        name: String::from("points"),
        query: String::from("SELECT geometry FROM osm_place_point LIMIT 1")
    }];
    let topics = vec![Topic {name: String::from("roads"), layers: layers}];
    let service = MvtService {input: pg, grid: grid, topics: topics};

    let mvt_tile = service.tile("roads", 486, 332, 10);
    //http://localhost:8124/roads/11/1073/717.pbf
    println!("{:#?}", mvt_tile);
    let expected = "Tile {
    layers: [
        Tile_Layer {
            version: Some(
                2
            ),
            name: Some(\"points\"),
            features: [
                Tile_Feature {
                    id: None,
                    tags: [],
                    field_type: Some(
                        POINT
                    ),
                    geometry: [
                        9,
                        405938,
                        214313
                    ],
                    unknown_fields: UnknownFields {
                        fields: None
                    },
                    cached_size: Cell { value: 0 }
                }
            ],
            keys: [],
            values: [],
            extent: Some(
                4096
            ),
            unknown_fields: UnknownFields {
                fields: None
            },
            cached_size: Cell { value: 0 }
        }
    ],
    unknown_fields: UnknownFields {
        fields: None
    },
    cached_size: Cell { value: 0 }
}";
    assert_eq!(expected, &*format!("{:#?}", mvt_tile));
}
