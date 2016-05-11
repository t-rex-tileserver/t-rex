use datasource::datasource::Datasource;
use datasource::postgis::PostgisInput;
use core::grid::{Extent,Grid};
use core::layer::Layer;
use mvt::tile::Tile;
use mvt::vector_tile;
use mvt::geom_to_proto::EncodableGeom;


/// Collection of layers in one MVT
pub struct Topic {
    pub name: String,
    pub layers: Vec<String>,
}

/// Mapbox Vector Tile Service
pub struct MvtService {
    pub input: PostgisInput,
    pub grid: Grid,
    pub layers: Vec<Layer>,
    pub topics: Vec<Topic>,
}

impl MvtService {
    fn get_layers(&self, name: &str) -> Vec<&Layer> {
        let topic = self.topics.iter().find(|t| t.name == name);
        match topic {
            Some(t) => Vec::new(), //TODO: return corresponding layers
            None => {
                self.layers.iter().filter(|t| t.name == name).collect()
            }
        }
    }
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let extent = self.grid.tile_extent_xyz(xtile, ytile, zoom);
        let mut tile = Tile::new(&extent, 4096);
        for layer in self.get_layers(topic).iter() {
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
    let mut layers = vec![Layer::new("points")];
    layers[0].table_name = Some(String::from("osm_place_point"));
    layers[0].geometry_field = Some(String::from("geometry"));
    layers[0].geometry_type = Some(String::from("POINT"));
    layers[0].query_limit = Some(1);
    let service = MvtService {input: pg, grid: grid, layers: layers, topics: Vec::new()};

    let mvt_tile = service.tile("points", 1073, 717, 11);
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
                        628,
                        5368
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
