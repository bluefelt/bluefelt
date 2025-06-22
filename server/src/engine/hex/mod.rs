// Hex Grid Engine Module
// Core hex grid functionality for Bluefelt

pub mod coordinates;
pub mod neighbors;
pub mod distance;
pub mod layout;
pub use coordinates::*;
pub use neighbors::*;
pub use distance::*;
pub use layout::*;

use serde::{Deserialize, Serialize};
use std::fmt;

/// Direction enum for hex grids
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HexDirection {
    East,      // 0
    Southeast, // 1
    Southwest, // 2
    West,      // 3
    Northwest, // 4
    Northeast, // 5
}

impl HexDirection {
    /// Get all six hex directions
    pub fn all() -> [HexDirection; 6] {
        [
            HexDirection::East,
            HexDirection::Southeast,
            HexDirection::Southwest,
            HexDirection::West,
            HexDirection::Northwest,
            HexDirection::Northeast,
        ]
    }

    /// Get the opposite direction
    pub fn opposite(self) -> HexDirection {
        match self {
            HexDirection::East => HexDirection::West,
            HexDirection::Southeast => HexDirection::Northwest,
            HexDirection::Southwest => HexDirection::Northeast,
            HexDirection::West => HexDirection::East,
            HexDirection::Northwest => HexDirection::Southeast,
            HexDirection::Northeast => HexDirection::Southwest,
        }
    }

    /// Get direction as integer (0-5)
    pub fn as_int(self) -> usize {
        match self {
            HexDirection::East => 0,
            HexDirection::Southeast => 1,
            HexDirection::Southwest => 2,
            HexDirection::West => 3,
            HexDirection::Northwest => 4,
            HexDirection::Northeast => 5,
        }
    }

    /// Create direction from integer (0-5)
    pub fn from_int(value: usize) -> Option<HexDirection> {
        match value % 6 {
            0 => Some(HexDirection::East),
            1 => Some(HexDirection::Southeast),
            2 => Some(HexDirection::Southwest),
            3 => Some(HexDirection::West),
            4 => Some(HexDirection::Northwest),
            5 => Some(HexDirection::Northeast),
            _ => None,
        }
    }
}

/// Hex layout type - flat or pointy top
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HexLayout {
    Flat,   // Flat top (⬢)
    Pointy, // Pointy top (⬡)
}

impl fmt::Display for HexLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HexLayout::Flat => write!(f, "flat"),
            HexLayout::Pointy => write!(f, "pointy"),
        }
    }
}

impl Default for HexLayout {
    fn default() -> Self {
        HexLayout::Flat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_direction_opposite() {
        assert_eq!(HexDirection::East.opposite(), HexDirection::West);
        assert_eq!(HexDirection::Southeast.opposite(), HexDirection::Northwest);
        assert_eq!(HexDirection::Southwest.opposite(), HexDirection::Northeast);
    }

    #[test]
    fn test_hex_direction_conversions() {
        for (i, dir) in HexDirection::all().iter().enumerate() {
            assert_eq!(dir.as_int(), i);
            assert_eq!(HexDirection::from_int(i), Some(*dir));
        }
    }

    #[test]
    fn test_hex_direction_wraparound() {
        assert_eq!(HexDirection::from_int(6), Some(HexDirection::East));
        assert_eq!(HexDirection::from_int(7), Some(HexDirection::Southeast));
    }
}