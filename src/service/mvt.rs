use service::postgis_input::PostgisInput;
use core::grid::{Extent,Grid};

/// Mapbox Vector Tile Service
pub struct MvtService {
    input: PostgisInput,
    grid: Grid
}

impl MvtService {
    /// Extent of a given tile in the grid given its x, y, and z
    pub fn tile_extent(&self, xtile: u16, ytile: u16, zoom: u16) -> Extent {
        self.grid.tile_extent_xyz(xtile, ytile, zoom)
    }
}

#[test]
pub fn test_tile_extent() {
    let pg = PostgisInput {connection_url: "postgresql://pi@%2Frun%2Fpostgresql/osm2vectortiles"};
    let grid = Grid::gmercator();
    let service = MvtService {input: pg, grid: grid};

    let extent = service.tile_extent(486, 332, 10);
    assert_eq!(extent, Extent {minx: -1017529.7205322683, miny: 7005300.768279828, maxx: -978393.9620502591, maxy: 7044436.526761841});
    //http://localhost:8124/roads/11/1073/717.pbf
    //let extent_ch = service.tile_extent(1073, 717, 11);
    //assert_eq!(extent_ch, Extent { minx: 958826.0828092434, miny: 5987771.04774756, maxx: 978393.9620502479, maxy: 6007338.926988564 });
}
