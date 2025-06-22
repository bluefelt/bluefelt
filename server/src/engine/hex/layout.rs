// Hex Layout and Rendering Support
// Convert between hex coordinates and pixel coordinates

use super::{HexCoord, HexLayout};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

/// Point in 2D space (for pixel coordinates)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Hex layout configuration for rendering
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HexLayoutConfig {
    pub layout: HexLayout,
    pub size: Point,    // Hex size (width/height scaling)
    pub origin: Point,  // Origin point in pixel space
}

impl Default for HexLayoutConfig {
    fn default() -> Self {
        Self {
            layout: HexLayout::Flat,
            size: Point::new(1.0, 1.0),
            origin: Point::new(0.0, 0.0),
        }
    }
}

impl HexLayoutConfig {
    /// Create new layout configuration
    pub fn new(layout: HexLayout, size: Point, origin: Point) -> Self {
        Self { layout, size, origin }
    }

    /// Create layout with uniform size
    pub fn with_size(layout: HexLayout, size: f32) -> Self {
        Self {
            layout,
            size: Point::new(size, size),
            origin: Point::new(0.0, 0.0),
        }
    }

    /// Convert hex coordinate to pixel coordinate
    pub fn hex_to_pixel(self, hex: HexCoord) -> Point {
        let matrix = match self.layout {
            HexLayout::Flat => FLAT_ORIENTATION,
            HexLayout::Pointy => POINTY_ORIENTATION,
        };

        let x = (matrix.f0 * hex.q as f32 + matrix.f1 * hex.r as f32) * self.size.x;
        let y = (matrix.f2 * hex.q as f32 + matrix.f3 * hex.r as f32) * self.size.y;

        Point::new(x + self.origin.x, y + self.origin.y)
    }

    /// Convert pixel coordinate to hex coordinate
    pub fn pixel_to_hex(self, point: Point) -> HexCoord {
        let matrix = match self.layout {
            HexLayout::Flat => FLAT_ORIENTATION,
            HexLayout::Pointy => POINTY_ORIENTATION,
        };

        let pt = Point::new(
            (point.x - self.origin.x) / self.size.x,
            (point.y - self.origin.y) / self.size.y,
        );

        let q = matrix.b0 * pt.x + matrix.b1 * pt.y;
        let r = matrix.b2 * pt.x + matrix.b3 * pt.y;

        hex_round(q, r)
    }

    /// Get corner points of a hex in pixel coordinates
    pub fn hex_corners(self, hex: HexCoord) -> [Point; 6] {
        let center = self.hex_to_pixel(hex);
        let mut corners = [Point::new(0.0, 0.0); 6];

        for i in 0..6 {
            let corner = self.hex_corner_offset(i);
            corners[i] = Point::new(center.x + corner.x, center.y + corner.y);
        }

        corners
    }

    /// Get offset for a corner relative to hex center
    fn hex_corner_offset(self, corner: usize) -> Point {
        let angle = 2.0 * PI * (self.start_angle() + corner as f32) / 6.0;
        Point::new(
            self.size.x * angle.cos(),
            self.size.y * angle.sin(),
        )
    }

    /// Get starting angle for corners based on layout
    fn start_angle(self) -> f32 {
        match self.layout {
            HexLayout::Flat => 0.0,
            HexLayout::Pointy => 0.5,
        }
    }

    /// Calculate bounding box for a set of hexes
    pub fn bounding_box(self, hexes: &[HexCoord]) -> Option<(Point, Point)> {
        if hexes.is_empty() {
            return None;
        }

        let pixels: Vec<Point> = hexes.iter().map(|&hex| self.hex_to_pixel(hex)).collect();

        let min_x = pixels.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
        let max_x = pixels.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
        let min_y = pixels.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
        let max_y = pixels.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);

        Some((Point::new(min_x, min_y), Point::new(max_x, max_y)))
    }
}

/// Orientation matrix for hex-to-pixel conversion
#[derive(Debug, Clone, Copy)]
struct Orientation {
    f0: f32, f1: f32, f2: f32, f3: f32, // forward matrix
    b0: f32, b1: f32, b2: f32, b3: f32, // backward matrix
}

// Math constants for hex layouts
const SQRT_3: f32 = 1.7320508;

const FLAT_ORIENTATION: Orientation = Orientation {
    f0: 3.0 / 2.0,
    f1: 0.0,
    f2: SQRT_3 / 2.0,
    f3: SQRT_3,
    b0: 2.0 / 3.0,
    b1: 0.0,
    b2: -1.0 / 3.0,
    b3: SQRT_3 / 3.0,
};

const POINTY_ORIENTATION: Orientation = Orientation {
    f0: SQRT_3,
    f1: SQRT_3 / 2.0,
    f2: 0.0,
    f3: 3.0 / 2.0,
    b0: SQRT_3 / 3.0,
    b1: -1.0 / 3.0,
    b2: 0.0,
    b3: 2.0 / 3.0,
};

/// Round fractional hex coordinates to nearest integer hex coordinates
fn hex_round(q: f32, r: f32) -> HexCoord {
    let s = -q - r;
    let mut rq = q.round();
    let mut rr = r.round();
    let mut rs = s.round();

    let q_diff = (rq - q).abs();
    let r_diff = (rr - r).abs();
    let s_diff = (rs - s).abs();

    if q_diff > r_diff && q_diff > s_diff {
        rq = -rr - rs;
    } else if r_diff > s_diff {
        rr = -rq - rs;
    } else {
        rs = -rq - rr;
    }

    HexCoord::new(rq as i32, rr as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_pixel_flat() {
        let layout = HexLayoutConfig::with_size(HexLayout::Flat, 1.0);
        
        let origin_pixel = layout.hex_to_pixel(HexCoord::origin());
        assert_eq!(origin_pixel.x, 0.0);
        assert_eq!(origin_pixel.y, 0.0);
        
        // Test neighbor conversion
        let east = layout.hex_to_pixel(HexCoord::new(1, 0));
        assert!((east.x - 1.5).abs() < 0.001);
        assert!((east.y - SQRT_3 / 2.0).abs() < 0.001);
    }

    #[test]
    fn test_hex_to_pixel_pointy() {
        let layout = HexLayoutConfig::with_size(HexLayout::Pointy, 1.0);
        
        let origin_pixel = layout.hex_to_pixel(HexCoord::origin());
        assert_eq!(origin_pixel.x, 0.0);
        assert_eq!(origin_pixel.y, 0.0);
        
        // Test neighbor conversion
        let east = layout.hex_to_pixel(HexCoord::new(1, 0));
        assert!((east.x - SQRT_3).abs() < 0.001);
        assert!(east.y.abs() < 0.001);
    }

    #[test]
    fn test_pixel_to_hex_round_trip() {
        let layout = HexLayoutConfig::with_size(HexLayout::Flat, 10.0);
        
        let original = HexCoord::new(2, -1);
        let pixel = layout.hex_to_pixel(original);
        let back_to_hex = layout.pixel_to_hex(pixel);
        
        assert_eq!(original, back_to_hex);
    }

    #[test]
    fn test_hex_corners() {
        let layout = HexLayoutConfig::with_size(HexLayout::Flat, 1.0);
        let corners = layout.hex_corners(HexCoord::origin());
        
        // Should have 6 corners
        assert_eq!(corners.len(), 6);
        
        // All corners should be at distance 1 from center
        for corner in corners {
            let distance = (corner.x * corner.x + corner.y * corner.y).sqrt();
            assert!((distance - 1.0).abs() < 0.001);
        }
    }

    #[test]
    fn test_bounding_box() {
        let layout = HexLayoutConfig::with_size(HexLayout::Flat, 1.0);
        let hexes = vec![
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            HexCoord::new(0, 2),
        ];
        
        let bbox = layout.bounding_box(&hexes).unwrap();
        assert!(bbox.0.x <= 0.0); // min_x should include origin
        assert!(bbox.1.x >= 3.0); // max_x should include (2,0)
        assert!(bbox.0.y <= 0.0); // min_y should include origin
        assert!(bbox.1.y >= 3.0); // max_y should include (0,2)
    }

    #[test]
    fn test_empty_bounding_box() {
        let layout = HexLayoutConfig::with_size(HexLayout::Flat, 1.0);
        let bbox = layout.bounding_box(&[]);
        assert!(bbox.is_none());
    }

    #[test]
    fn test_hex_round() {
        // Test exact coordinates
        assert_eq!(hex_round(1.0, 0.0), HexCoord::new(1, 0));
        assert_eq!(hex_round(0.0, 1.0), HexCoord::new(0, 1));
        
        // Test fractional coordinates that should round
        assert_eq!(hex_round(1.2, -0.1), HexCoord::new(1, 0));
        assert_eq!(hex_round(0.8, 0.3), HexCoord::new(1, 0));
    }

    #[test]
    fn test_layout_with_origin_offset() {
        let layout = HexLayoutConfig::new(
            HexLayout::Flat,
            Point::new(1.0, 1.0),
            Point::new(100.0, 200.0),
        );
        
        let pixel = layout.hex_to_pixel(HexCoord::origin());
        assert_eq!(pixel.x, 100.0);
        assert_eq!(pixel.y, 200.0);
    }

    #[test]
    fn test_layout_with_size_scaling() {
        let layout = HexLayoutConfig::new(
            HexLayout::Flat,
            Point::new(10.0, 20.0),
            Point::new(0.0, 0.0),
        );
        
        let pixel = layout.hex_to_pixel(HexCoord::new(1, 0));
        assert!((pixel.x - 15.0).abs() < 0.001); // 1.5 * 10.0
        assert!((pixel.y - SQRT_3 / 2.0 * 20.0).abs() < 0.001);
    }
}