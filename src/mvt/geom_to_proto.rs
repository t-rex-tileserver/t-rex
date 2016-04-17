//! Encode geometries according to MVT spec
//! https://github.com/mapbox/vector-tile-spec/tree/master/2.1

use std::vec::Vec;
//https://github.com/andelf/rust-postgis
use postgis::{Point,SRID,WGS84};

#[allow(missing_copy_implementations)]
#[allow(non_camel_case_types)]
pub enum EPSG_3857 {}

impl SRID for EPSG_3857 {
    fn as_srid() -> Option<i32> { Some(3857) }
}

/// Command to be executed and the number of times that the command will be executed
/// https://github.com/mapbox/vector-tile-spec/tree/master/2.1#431-command-integers
pub struct CommandInteger(u32);

pub enum Command {
    MoveTo    = 1,
    LineTo    = 2,
    ClosePath = 7,
}

impl CommandInteger {
    pub fn new(id: Command, count: u32) -> CommandInteger {
        CommandInteger(((id as u32) & 0x7) | (count << 3))
    }

    pub fn id(&self) -> u32 {
        self.0 & 0x7
    }

    pub fn count(&self) -> u32 {
        self.0 >> 3
    }
}

#[test]
fn test_commands() {
    assert_eq!(CommandInteger(9).id(), Command::MoveTo as u32);
    assert_eq!(CommandInteger(9).count(), 1);

    assert_eq!(CommandInteger::new(Command::MoveTo, 1).0, 9);
    assert_eq!(CommandInteger::new(Command::LineTo, 3).0, 26);
    assert_eq!(CommandInteger::new(Command::ClosePath, 1).0, 15);
}


/// Commands requiring parameters are followed by a ParameterInteger for each parameter required by that command
/// https://github.com/mapbox/vector-tile-spec/tree/master/2.1#432-parameter-integers
pub struct ParameterInteger(u32);

impl ParameterInteger {
    pub fn new(value: i32) -> ParameterInteger {
        ParameterInteger(((value << 1) ^ (value >> 31)) as u32)
    }

    pub fn value(&self) -> i32 {
        ((self.0 >> 1) as i32) ^ (-((self.0 & 1) as i32))
    }
}

#[test]
fn test_paremeters() {
    assert_eq!(ParameterInteger(50).value(), 25);
    assert_eq!(ParameterInteger::new(25).value(), 25);
}


pub struct CommandSequence(Vec<u32>);

impl CommandSequence {
    pub fn new() -> CommandSequence {
        CommandSequence(Vec::new())
    }
    pub fn append(&mut self, other: &mut CommandSequence) {
        self.0.append(&mut other.0);
    }
    pub fn push(&mut self, value: u32) {
        self.0.push(value);
    }
}

// Geometry types in screen coordinates
#[derive(PartialEq,Debug)]
struct PointScreen {
    x: i32,
    y: i32
}

impl PointScreen {
    pub fn encode(&self) -> CommandSequence {
        CommandSequence(vec![
            CommandInteger::new(Command::MoveTo, 1).0,
            ParameterInteger::new(self.x).0,
            ParameterInteger::new(self.y).0
        ])        
    }
}

#[derive(PartialEq,Debug)]
struct MultiPointScreen {
    points: Vec<PointScreen>
}

impl MultiPointScreen {
    pub fn encode(&self) -> CommandSequence {
        let mut seq = CommandSequence::new();
        seq.push(CommandInteger::new(
            Command::MoveTo, self.points.len() as u32).0);
        let (mut posx, mut posy) = (0, 0);
        for point in &self.points {
            seq.push(ParameterInteger::new(point.x - posx).0);
            seq.push(ParameterInteger::new(point.y - posy).0);
            posx = point.x;
            posy = point.y;
        }
        seq
    }
}

#[derive(PartialEq,Debug)]
struct LinestringScreen {
    points: Vec<PointScreen>
}

impl LinestringScreen {
    pub fn encode(&self) -> CommandSequence {
        let mut seq = CommandSequence::new();
        if self.points.len() > 0 {
            seq = self.points[0].encode();
            seq.push(CommandInteger::new(
                Command::LineTo, (self.points.len()-1) as u32).0);
            for i in 1..self.points.len() {
                let ref pos = &self.points[i-1];
                let ref point = &self.points[i];
                seq.push(ParameterInteger::new(point.x - pos.x).0);
                seq.push(ParameterInteger::new(point.y - pos.y).0);
            }
        }
        seq
    }
}

#[test]
fn test_geom_encoding() {
    let point = PointScreen { x: 25, y: 17 };
    assert_eq!(point.encode().0, &[9,50,34]);

    let multipoint = MultiPointScreen {
        points: vec![
            PointScreen { x: 5, y: 7 },
            PointScreen { x: 3, y: 2 }
            ]
        };
    assert_eq!(multipoint.encode().0, &[17,10,14,3,9]);

    let linestring = LinestringScreen {
        points: vec![
            PointScreen { x: 2, y: 2 },
            PointScreen { x: 2, y: 10 },
            PointScreen { x: 10, y: 10 }
            ]
        };
    assert_eq!(linestring.encode().0, &[9,4,4,18,0,16,16,0]);
}


#[derive(PartialEq,Debug)]
struct Tilebound {
    x0: f64,
    y0: f64,
    x_max: f64,
    y_max: f64,
}

/// Convert geometry to tile relative coordinates
fn geom_in_tile(tile_bounds: Tilebound, tile_size: u32, geom: Point<EPSG_3857>) -> PointScreen {
    let x_span = tile_bounds.x_max - tile_bounds.x0;
    let y_span = tile_bounds.y_max - tile_bounds.y0;
    PointScreen {
        x: ((geom.x-tile_bounds.x0) * tile_size as f64 / x_span) as i32,
        y: ((geom.y-tile_bounds.y0) * tile_size as f64 / y_span) as i32 }
}

#[test]
fn test_sfgeom_encoding() {
    //let zh_mercator = Point::<EPSG_3857>::new(949398.0, 6002729.0);
    let zh_mercator = Point::<EPSG_3857>::new(960000.0, 6002729.0);
    //let zh_wgs84 = Point::<WGS84>::new(47.3703149, 8.5285874);
    let screen_pt = geom_in_tile(
        Tilebound {x0: 958826.08, y0: 5987771.04, x_max: 978393.96, y_max: 6007338.92},
        4096, zh_mercator);
    assert_eq!(screen_pt, PointScreen { x: 245, y: 3131 });
    assert_eq!(screen_pt.encode().0, &[9,490,6262]);
}
