//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

use crate::core::config::GridCfg;
use crate::core::Config;
use tile_grid::{Extent, Grid, Origin, Unit};

#[derive(Deserialize, Clone, Debug)]
pub struct ExtentCfg {
    pub minx: f64,
    pub miny: f64,
    pub maxx: f64,
    pub maxy: f64,
}

impl From<&ExtentCfg> for Extent {
    fn from(cfg: &ExtentCfg) -> Extent {
        Extent {
            minx: cfg.minx,
            miny: cfg.miny,
            maxx: cfg.maxx,
            maxy: cfg.maxy,
        }
    }
}

impl<'a> Config<'a, GridCfg> for Grid {
    fn from_config(grid_cfg: &GridCfg) -> Result<Self, String> {
        if let Some(ref gridname) = grid_cfg.predefined {
            match gridname.as_str() {
                "wgs84" => Ok(Grid::wgs84()),
                "web_mercator" => Ok(Grid::web_mercator()),
                _ => Err(format!("Unkown grid '{}'", gridname)),
            }
        } else if let Some(ref usergrid) = grid_cfg.user {
            let units = match &usergrid.units.to_lowercase() as &str {
                "m" => Ok(Unit::Meters),
                "dd" => Ok(Unit::Degrees),
                "ft" => Ok(Unit::Feet),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.units)),
            };
            let origin = match &usergrid.origin as &str {
                "TopLeft" => Ok(Origin::TopLeft),
                "BottomLeft" => Ok(Origin::BottomLeft),
                _ => Err(format!("Unexpected enum value '{}'", usergrid.origin)),
            };
            let grid = Grid::new(
                usergrid.width,
                usergrid.height,
                Extent::from(&usergrid.extent),
                usergrid.srid,
                units?,
                usergrid.resolutions.clone(),
                origin?,
            );
            Ok(grid)
        } else {
            Err("Invalid grid definition".to_string())
        }
    }
    fn gen_config() -> String {
        let toml = r#"
[grid]
predefined = "web_mercator"
"#;
        toml.to_string()
    }
}
