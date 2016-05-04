use datasource::postgis::PostgisInput;
use core::grid::{Extent,Grid};
use mvt::tile::Tile;
use mvt::vector_tile;
use mvt::geom_to_proto::EncodableGeom;

/// Mapbox Vector Tile Service
pub struct MvtService {
    input: PostgisInput,
    grid: Grid
}

impl MvtService {
    /// Create vector tile from input at x, y, z
    pub fn tile(&self, topic: &str, xtile: u16, ytile: u16, zoom: u16) -> vector_tile::Tile {
        let extent = self.grid.tile_extent_xyz(xtile, ytile, zoom);
        let tile = Tile::new(&extent, 4096);
        /*
        for layer in topic.layers().iter() {
            let mvt_layer = tile.new_layer(layer);
            for feature in self.input.retrieve_features(&layer, &extent, zoom) {
                let mvt_feature = tile.new_feature(feature);
                let mvt_geom = tile.encode_geom(feature.geom());
                mvt_feature.set_geometry(geom);
                mvt_layer.mut_features().push(mvt_feature);
            }
            tile.add_layer(mvt_layer);
        }
        */
        tile.mvt_tile
    }
}

#[test]
pub fn test_tile_query() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let grid = Grid::web_mercator();
    let service = MvtService {input: pg, grid: grid};

    let mvt = service.tile("roads", 486, 332, 10);
    //http://localhost:8124/roads/11/1073/717.pbf
    assert_eq!(mvt, vector_tile::Tile::new());
}
