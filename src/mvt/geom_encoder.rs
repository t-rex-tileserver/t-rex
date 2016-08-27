//
// Copyright (c) Pirmin Kalberer. All rights reserved.
// Licensed under the MIT License. See LICENSE file in the project root for full license information.
//

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
    #[cfg(test)]
    fn id(&self) -> u32 {
        self.0 & 0x7
    }
    #[cfg(test)]
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
    #[cfg(test)]
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
    pub fn vec(&self) -> Vec<u32> {
        self.0.clone() // FIXME: ref
    }
    #[cfg(test)]
    fn append(&mut self, other: &mut CommandSequence) {
        self.0.append(&mut other.0);
    }
    fn push(&mut self, value: u32) {
        self.0.push(value);
    }
}

#[test]
fn test_sequence() {
    let mut seq = CommandSequence::new();
    seq.push(CommandInteger::new(
        Command::MoveTo, 1).0);
    seq.push(ParameterInteger::new(25).0);
    seq.push(ParameterInteger::new(17).0);
    assert_eq!(seq.0, &[9,50,34]);

    let mut seq2 = CommandSequence::new();
    seq2.push(CommandInteger::new(
        Command::MoveTo, 1).0);
    seq.append(&mut seq2);
    assert_eq!(seq.0, &[9,50,34,9]);
}

pub trait EncodableGeom {
    fn encode(&self) -> CommandSequence {
        let mut seq = CommandSequence::new();
        self.encode_from(&screen::Point::origin(), &mut seq);
        seq
    }
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence);
}

impl EncodableGeom for screen::Point {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        seq.push(CommandInteger::new(
            Command::MoveTo, 1).0);
        seq.push(ParameterInteger::new(self.x - startpos.x).0);
        seq.push(ParameterInteger::new(self.y - startpos.y).0);
    }
}

impl EncodableGeom for screen::MultiPoint {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        seq.push(CommandInteger::new(
            Command::MoveTo, self.points.len() as u32).0);
        let (mut posx, mut posy) = (startpos.x, startpos.y);
        for point in &self.points {
            seq.push(ParameterInteger::new(point.x - posx).0);
            seq.push(ParameterInteger::new(point.y - posy).0);
            posx = point.x;
            posy = point.y;
        }
    }
}

impl EncodableGeom for screen::LineString {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        if self.points.len() > 0 {
            self.points[0].encode_from(startpos, seq);
            seq.push(CommandInteger::new(
                Command::LineTo, (self.points.len()-1) as u32).0);
            for i in 1..self.points.len() {
                let ref pos = &self.points[i-1];
                let ref point = &self.points[i];
                seq.push(ParameterInteger::new(point.x - pos.x).0);
                seq.push(ParameterInteger::new(point.y - pos.y).0);
            }
        }
    }
}
impl screen::LineString {
    fn encode_ring_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        // almost same as LineString.encode_from, with ClosePath instead of last point
        if self.points.len() > 0 {
            self.points[0].encode_from(startpos, seq);
            seq.push(CommandInteger::new(
                Command::LineTo, (self.points.len()-2) as u32).0);
            for i in 1..self.points.len()-1 {
                let ref pos = &self.points[i-1];
                let ref point = &self.points[i];
                seq.push(ParameterInteger::new(point.x - pos.x).0);
                seq.push(ParameterInteger::new(point.y - pos.y).0);
            }
            seq.push(CommandInteger::new(Command::ClosePath, 1).0);
        }
    }
}

impl EncodableGeom for screen::MultiLineString {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for line in &self.lines {
            if line.points.len() > 0 {
                line.encode_from(&pos, seq);
                pos = &line.points[line.points.len()-1];
            }
        }
    }
}

impl EncodableGeom for screen::Polygon {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for line in &self.rings {
            if line.points.len() > 1 {
                line.encode_ring_from(&pos, seq);
                pos = &line.points[line.points.len()-2];
            }
        }
    }
}

impl EncodableGeom for screen::MultiPolygon {
    fn encode_from(&self, startpos: &screen::Point, seq: &mut CommandSequence) {
        let mut pos = startpos;
        for polygon in &self.polygons {
            for line in &polygon.rings {
                if line.points.len() > 1 {
                    line.encode_ring_from(&pos, seq);
                    pos = &line.points[line.points.len()-2];
                }
            }
        }
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

    let linestring = screen::LineString {
        points: vec![
            screen::Point { x: 2, y: 2 },
            screen::Point { x: 2, y: 10 },
            screen::Point { x: 10, y: 10 }
            ]
        };
    assert_eq!(linestring.encode().0, &[9,4,4,18,0,16,16,0]);

    let multilinestring = screen::MultiLineString {
        lines: vec![
            screen::LineString {
                points: vec![
                    screen::Point { x: 2, y: 2 },
                    screen::Point { x: 2, y: 10 },
                    screen::Point { x: 10, y: 10 }
                    ]
            },
            screen::LineString {
                points: vec![
                    screen::Point { x: 1, y: 1 },
                    screen::Point { x: 3, y: 5 }
                    ]
            }
            ]
        };
    assert_eq!(multilinestring.encode().0, &[9,4,4,18,0,16,16,0,9,17,17,10,4,8]);

    let polygon = screen::Polygon {
        rings: vec![
            screen::LineString {
                points: vec![
                    screen::Point { x: 3, y: 6 },
                    screen::Point { x: 8, y: 12 },
                    screen::Point { x: 20, y: 34 },
                    screen::Point { x: 3, y: 6 }
                    ]
            }
            ]
        };
    assert_eq!(polygon.encode().0, &[9,6,12,18,10,12,24,44,15]);

    let multipolygon = screen::MultiPolygon {
        polygons: vec![
            screen::Polygon {
                rings: vec![
                    screen::LineString {
                        points: vec![
                            screen::Point { x: 0, y: 0 },
                            screen::Point { x: 10, y: 0 },
                            screen::Point { x: 10, y: 10 },
                            screen::Point { x: 0, y: 10 },
                            screen::Point { x: 0, y: 0 }
                            ]
                    }
                    ]
            },
            screen::Polygon {
                rings: vec![
                    screen::LineString {
                        points: vec![
                            screen::Point { x: 11, y: 11 },
                            screen::Point { x: 20, y: 11 },
                            screen::Point { x: 20, y: 20 },
                            screen::Point { x: 11, y: 20 },
                            screen::Point { x: 11, y: 20 },
                            screen::Point { x: 11, y: 11 }
                            ]
                    },
                    screen::LineString {
                        points: vec![
                            screen::Point { x: 13, y: 13 },
                            screen::Point { x: 13, y: 17 },
                            screen::Point { x: 17, y: 17 },
                            screen::Point { x: 17, y: 13 },
                            screen::Point { x: 13, y: 13 }
                            ]
                    }
                    ]
            }
            ]
        };
    let expected = [9,0,0,26,20,0,0,20,19,0,15,9,22,2,34,18,0,0,18,17,0,0,0,15,9,4,13,26,0,8,8,0,0,7,15];
    assert_eq!(multipolygon.encode().0, &expected[0..35]);
}
