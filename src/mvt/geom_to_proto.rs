//! Encode geometries according to MVT spec
//! https://github.com/mapbox/vector-tile-spec/tree/master/2.1

use std::vec::Vec;
use core::screen;

/// Command to be executed and the number of times that the command will be executed
/// https://github.com/mapbox/vector-tile-spec/tree/master/2.1#431-command-integers
struct CommandInteger(u32);

enum Command {
    MoveTo    = 1,
    LineTo    = 2,
    ClosePath = 7,
}

impl CommandInteger {
    fn new(id: Command, count: u32) -> CommandInteger {
        CommandInteger(((id as u32) & 0x7) | (count << 3))
    }

    fn id(&self) -> u32 {
        self.0 & 0x7
    }

    fn count(&self) -> u32 {
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
struct ParameterInteger(u32);

impl ParameterInteger {
    fn new(value: i32) -> ParameterInteger {
        ParameterInteger(((value << 1) ^ (value >> 31)) as u32)
    }

    fn value(&self) -> i32 {
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
    fn new() -> CommandSequence {
        CommandSequence(Vec::new())
    }
    fn append(&mut self, other: &mut CommandSequence) {
        self.0.append(&mut other.0);
    }
    fn push(&mut self, value: u32) {
        self.0.push(value);
    }
}


pub trait EncodableGeom {
    fn encode(&self) -> CommandSequence;
}

impl EncodableGeom for screen::Point {
    fn encode(&self) -> CommandSequence {
        CommandSequence(vec![
            CommandInteger::new(Command::MoveTo, 1).0,
            ParameterInteger::new(self.x).0,
            ParameterInteger::new(self.y).0
        ])        
    }
}

impl EncodableGeom for screen::MultiPoint {
    fn encode(&self) -> CommandSequence {
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

impl EncodableGeom for screen::Linestring {
    fn encode(&self) -> CommandSequence {
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
    let point = screen::Point { x: 25, y: 17 };
    assert_eq!(point.encode().0, &[9,50,34]);

    let multipoint = screen::MultiPoint {
        points: vec![
            screen::Point { x: 5, y: 7 },
            screen::Point { x: 3, y: 2 }
            ]
        };
    assert_eq!(multipoint.encode().0, &[17,10,14,3,9]);

    let linestring = screen::Linestring {
        points: vec![
            screen::Point { x: 2, y: 2 },
            screen::Point { x: 2, y: 10 },
            screen::Point { x: 10, y: 10 }
            ]
        };
    assert_eq!(linestring.encode().0, &[9,4,4,18,0,16,16,0]);
}
