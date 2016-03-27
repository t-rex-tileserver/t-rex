//! Encode geometries according to MVT spec
//! https://github.com/mapbox/vector-tile-spec/tree/master/2.1

use std::vec::Vec;

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
        let uval = value as u32; 
        ParameterInteger((uval << 1) ^ (uval >> 31))
    }

    pub fn value(&self) -> i32 {
        let sval = self.0 as i32; 
        ((sval >> 1) ^ (-(sval & 1)))
    }
}

#[test]
fn test_paremeters() {
    assert_eq!(ParameterInteger(50).value(), 25);
    assert_eq!(ParameterInteger::new(25).value(), 25);
}


pub struct CommandSequence(Vec<u32>);


// Geometry types in screen coordinates

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

#[test]
fn test_geom_encoding() {
    let point = PointScreen { x: 25, y: 17 };
    assert_eq!(point.encode().0, &[9,50,34]);
}
