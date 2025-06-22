// Hex Coordinate Systems
// Support for axial, cube, and offset coordinate systems

use serde::{Deserialize, Serialize};
use std::fmt;
use super::HexLayout;

/// Axial coordinates (q, r) - Primary coordinate system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AxialCoord {
    pub q: i32,
    pub r: i32,
}

/// Cube coordinates (x, y, z) where x + y + z = 0
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CubeCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Offset coordinates (col, row) - Traditional grid coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OffsetCoord {
    pub col: i32,
    pub row: i32,
}

/// Main hex coordinate type - uses axial internally
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

impl HexCoord {
    /// Create new hex coordinate
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Create from axial coordinates
    pub fn from_axial(axial: AxialCoord) -> Self {
        Self { q: axial.q, r: axial.r }
    }

    /// Create from cube coordinates
    pub fn from_cube(cube: CubeCoord) -> Self {
        assert_eq!(cube.x + cube.y + cube.z, 0, "Invalid cube coordinates");
        Self { q: cube.x, r: cube.z }
    }

    /// Create from offset coordinates
    pub fn from_offset(offset: OffsetCoord, layout: HexLayout) -> Self {
        match layout {
            HexLayout::Flat => {
                // Odd-q vertical layout
                let q = offset.col;
                let r = offset.row - (offset.col - (offset.col & 1)) / 2;
                Self { q, r }
            }
            HexLayout::Pointy => {
                // Odd-r horizontal layout
                let q = offset.col - (offset.row - (offset.row & 1)) / 2;
                let r = offset.row;
                Self { q, r }
            }
        }
    }

    /// Convert to axial coordinates
    pub fn to_axial(self) -> AxialCoord {
        AxialCoord { q: self.q, r: self.r }
    }

    /// Convert to cube coordinates
    pub fn to_cube(self) -> CubeCoord {
        CubeCoord {
            x: self.q,
            y: -self.q - self.r,
            z: self.r,
        }
    }

    /// Convert to offset coordinates
    pub fn to_offset(self, layout: HexLayout) -> OffsetCoord {
        match layout {
            HexLayout::Flat => {
                // Odd-q vertical layout
                let col = self.q;
                let row = self.r + (self.q - (self.q & 1)) / 2;
                OffsetCoord { col, row }
            }
            HexLayout::Pointy => {
                // Odd-r horizontal layout
                let col = self.q + (self.r - (self.r & 1)) / 2;
                let row = self.r;
                OffsetCoord { col, row }
            }
        }
    }

    /// Create hex coordinate from string representation "q,r"
    pub fn from_string(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid hex coordinate format: {}", s));
        }

        let q = parts[0].trim().parse::<i32>()
            .map_err(|_| format!("Invalid q coordinate: {}", parts[0]))?;
        let r = parts[1].trim().parse::<i32>()
            .map_err(|_| format!("Invalid r coordinate: {}", parts[1]))?;

        Ok(HexCoord::new(q, r))
    }

    /// Get the origin coordinate (0, 0)
    pub fn origin() -> Self {
        Self::new(0, 0)
    }
}

impl fmt::Display for HexCoord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.q, self.r)
    }
}

impl From<AxialCoord> for HexCoord {
    fn from(axial: AxialCoord) -> Self {
        HexCoord::from_axial(axial)
    }
}

impl From<CubeCoord> for HexCoord {
    fn from(cube: CubeCoord) -> Self {
        HexCoord::from_cube(cube)
    }
}

impl From<HexCoord> for AxialCoord {
    fn from(hex: HexCoord) -> Self {
        hex.to_axial()
    }
}

impl From<HexCoord> for CubeCoord {
    fn from(hex: HexCoord) -> Self {
        hex.to_cube()
    }
}

// Arithmetic operations for hex coordinates
impl std::ops::Add for HexCoord {
    type Output = Self;
    
    fn add(self, other: Self) -> Self {
        HexCoord::new(self.q + other.q, self.r + other.r)
    }
}

impl std::ops::Sub for HexCoord {
    type Output = Self;
    
    fn sub(self, other: Self) -> Self {
        HexCoord::new(self.q - other.q, self.r - other.r)
    }
}

impl std::ops::Mul<i32> for HexCoord {
    type Output = Self;
    
    fn mul(self, scalar: i32) -> Self {
        HexCoord::new(self.q * scalar, self.r * scalar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversions() {
        let hex = HexCoord::new(1, 2);
        
        // Test cube conversion
        let cube = hex.to_cube();
        assert_eq!(cube.x, 1);
        assert_eq!(cube.y, -3);
        assert_eq!(cube.z, 2);
        assert_eq!(cube.x + cube.y + cube.z, 0);
        
        // Test round-trip conversion
        let back_to_hex = HexCoord::from_cube(cube);
        assert_eq!(hex, back_to_hex);
    }

    #[test]
    fn test_offset_conversion_flat() {
        let hex = HexCoord::new(1, 2);
        let offset = hex.to_offset(HexLayout::Flat);
        let back_to_hex = HexCoord::from_offset(offset, HexLayout::Flat);
        assert_eq!(hex, back_to_hex);
    }

    #[test]
    fn test_offset_conversion_pointy() {
        let hex = HexCoord::new(1, 2);
        let offset = hex.to_offset(HexLayout::Pointy);
        let back_to_hex = HexCoord::from_offset(offset, HexLayout::Pointy);
        assert_eq!(hex, back_to_hex);
    }

    #[test]
    fn test_hex_arithmetic() {
        let a = HexCoord::new(1, 2);
        let b = HexCoord::new(3, 1);
        
        assert_eq!(a + b, HexCoord::new(4, 3));
        assert_eq!(b - a, HexCoord::new(2, -1));
        assert_eq!(a * 2, HexCoord::new(2, 4));
    }

    #[test]
    fn test_string_conversion() {
        let hex = HexCoord::new(1, -2);
        assert_eq!(hex.to_string(), "1,-2");
        
        let parsed = HexCoord::from_string("1,-2").unwrap();
        assert_eq!(hex, parsed);
        
        let parsed = HexCoord::from_string(" 1 , -2 ").unwrap();
        assert_eq!(hex, parsed);
    }

    #[test]
    fn test_invalid_string_conversion() {
        assert!(HexCoord::from_string("invalid").is_err());
        assert!(HexCoord::from_string("1,2,3").is_err());
        assert!(HexCoord::from_string("a,b").is_err());
    }

    #[test]
    #[should_panic(expected = "Invalid cube coordinates")]
    fn test_invalid_cube_coordinates() {
        let _hex = HexCoord::from_cube(CubeCoord { x: 1, y: 2, z: 3 });
    }
}