// Hex Neighbor Operations
// Calculate neighbors, directions, and adjacency relationships

use super::{HexCoord, HexDirection};

/// Direction vectors for axial coordinates
const AXIAL_DIRECTIONS: [(i32, i32); 6] = [
    (1, 0),   // East
    (0, 1),   // Southeast
    (-1, 1),  // Southwest
    (-1, 0),  // West
    (0, -1),  // Northwest
    (1, -1),  // Northeast
];

impl HexCoord {
    /// Get neighbor in a specific direction
    pub fn neighbor(self, direction: HexDirection) -> HexCoord {
        let (dq, dr) = AXIAL_DIRECTIONS[direction.as_int()];
        HexCoord::new(self.q + dq, self.r + dr)
    }

    /// Get all six neighbors
    pub fn neighbors(self) -> [HexCoord; 6] {
        [
            self.neighbor(HexDirection::East),
            self.neighbor(HexDirection::Southeast),
            self.neighbor(HexDirection::Southwest),
            self.neighbor(HexDirection::West),
            self.neighbor(HexDirection::Northwest),
            self.neighbor(HexDirection::Northeast),
        ]
    }

    /// Get neighbors as a vector
    pub fn neighbors_vec(self) -> Vec<HexCoord> {
        self.neighbors().to_vec()
    }

    /// Get direction from this hex to another hex
    pub fn direction_to(self, other: HexCoord) -> Option<HexDirection> {
        let diff = other - self;
        
        for direction in HexDirection::all() {
            let (dq, dr) = AXIAL_DIRECTIONS[direction.as_int()];
            if diff.q == dq && diff.r == dr {
                return Some(direction);
            }
        }
        
        None
    }

    /// Check if two hexes are adjacent (neighbors)
    pub fn is_adjacent(self, other: HexCoord) -> bool {
        self.direction_to(other).is_some()
    }

    /// Get all hexes within a certain distance
    pub fn hexes_within_range(self, range: i32) -> Vec<HexCoord> {
        let mut results = Vec::new();
        
        for q in -range..=range {
            let r1 = std::cmp::max(-range, -q - range);
            let r2 = std::cmp::min(range, -q + range);
            
            for r in r1..=r2 {
                results.push(HexCoord::new(self.q + q, self.r + r));
            }
        }
        
        results
    }

    /// Get hexes at exactly a certain distance (ring)
    pub fn ring(self, radius: i32) -> Vec<HexCoord> {
        if radius == 0 {
            return vec![self];
        }
        
        let mut results = Vec::new();
        let mut hex = self + HexCoord::new(0, -radius); // Start at top
        
        for direction in HexDirection::all() {
            for _ in 0..radius {
                results.push(hex);
                hex = hex.neighbor(direction);
            }
        }
        
        results
    }

    /// Get all hexes in a line from this hex to another
    pub fn line_to(self, target: HexCoord) -> Vec<HexCoord> {
        let distance = self.distance(target);
        if distance == 0 {
            return vec![self];
        }
        
        let mut results = Vec::new();
        
        for i in 0..=distance {
            let t = i as f32 / distance as f32;
            let cube1 = self.to_cube();
            let cube2 = target.to_cube();
            
            // Linear interpolation in cube coordinates
            let x = cube1.x as f32 * (1.0 - t) + cube2.x as f32 * t;
            let y = cube1.y as f32 * (1.0 - t) + cube2.y as f32 * t;
            let z = cube1.z as f32 * (1.0 - t) + cube2.z as f32 * t;
            
            // Round to nearest cube coordinate
            let rounded = round_cube(x, y, z);
            results.push(HexCoord::from_cube(rounded));
        }
        
        results
    }

    /// Check if there's a clear line of sight to another hex
    pub fn has_line_of_sight(self, target: HexCoord, blocked: &[HexCoord]) -> bool {
        let line = self.line_to(target);
        
        // Check if any hex in the line (except start and end) is blocked
        for hex in line.iter().skip(1).take(line.len().saturating_sub(2)) {
            if blocked.contains(hex) {
                return false;
            }
        }
        
        true
    }
}

/// Round floating point cube coordinates to nearest integer cube coordinates
fn round_cube(x: f32, y: f32, z: f32) -> super::CubeCoord {
    let mut rx = x.round() as i32;
    let mut ry = y.round() as i32;
    let mut rz = z.round() as i32;
    
    let x_diff = (rx as f32 - x).abs();
    let y_diff = (ry as f32 - y).abs();
    let z_diff = (rz as f32 - z).abs();
    
    if x_diff > y_diff && x_diff > z_diff {
        rx = -ry - rz;
    } else if y_diff > z_diff {
        ry = -rx - rz;
    } else {
        rz = -rx - ry;
    }
    
    super::CubeCoord { x: rx, y: ry, z: rz }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_neighbors() {
        let center = HexCoord::origin();
        let neighbors = center.neighbors();
        
        assert_eq!(neighbors.len(), 6);
        assert_eq!(neighbors[0], HexCoord::new(1, 0));   // East
        assert_eq!(neighbors[1], HexCoord::new(0, 1));   // Southeast
        assert_eq!(neighbors[2], HexCoord::new(-1, 1));  // Southwest
        assert_eq!(neighbors[3], HexCoord::new(-1, 0));  // West
        assert_eq!(neighbors[4], HexCoord::new(0, -1));  // Northwest
        assert_eq!(neighbors[5], HexCoord::new(1, -1));  // Northeast
    }

    #[test]
    fn test_hex_direction_to() {
        let center = HexCoord::origin();
        
        assert_eq!(center.direction_to(HexCoord::new(1, 0)), Some(HexDirection::East));
        assert_eq!(center.direction_to(HexCoord::new(0, 1)), Some(HexDirection::Southeast));
        assert_eq!(center.direction_to(HexCoord::new(-1, 1)), Some(HexDirection::Southwest));
        assert_eq!(center.direction_to(HexCoord::new(-1, 0)), Some(HexDirection::West));
        assert_eq!(center.direction_to(HexCoord::new(0, -1)), Some(HexDirection::Northwest));
        assert_eq!(center.direction_to(HexCoord::new(1, -1)), Some(HexDirection::Northeast));
        
        // Non-adjacent hex should return None
        assert_eq!(center.direction_to(HexCoord::new(2, 0)), None);
    }

    #[test]
    fn test_hex_adjacency() {
        let center = HexCoord::origin();
        
        // All neighbors should be adjacent
        for neighbor in center.neighbors() {
            assert!(center.is_adjacent(neighbor));
            assert!(neighbor.is_adjacent(center));
        }
        
        // Non-neighbors should not be adjacent
        assert!(!center.is_adjacent(HexCoord::new(2, 0)));
        assert!(!center.is_adjacent(HexCoord::new(0, 2)));
    }

    #[test]
    fn test_hexes_within_range() {
        let center = HexCoord::origin();
        
        let range_0 = center.hexes_within_range(0);
        assert_eq!(range_0.len(), 1);
        assert!(range_0.contains(&center));
        
        let range_1 = center.hexes_within_range(1);
        assert_eq!(range_1.len(), 7); // center + 6 neighbors
        
        let range_2 = center.hexes_within_range(2);
        assert_eq!(range_2.len(), 19); // 1 + 6 + 12
    }

    #[test]
    fn test_hex_ring() {
        let center = HexCoord::origin();
        
        let ring_0 = center.ring(0);
        assert_eq!(ring_0.len(), 1);
        assert_eq!(ring_0[0], center);
        
        let ring_1 = center.ring(1);
        assert_eq!(ring_1.len(), 6);
        
        let ring_2 = center.ring(2);
        assert_eq!(ring_2.len(), 12);
        
        // Check that all hexes in ring are at correct distance
        for hex in ring_2 {
            assert_eq!(center.distance(hex), 2);
        }
    }

    #[test]
    fn test_hex_line() {
        let start = HexCoord::origin();
        let end = HexCoord::new(3, 0);
        
        let line = start.line_to(end);
        assert_eq!(line.len(), 4); // 0, 1, 2, 3
        assert_eq!(line[0], start);
        assert_eq!(line[3], end);
        
        // Check intermediate points
        assert_eq!(line[1], HexCoord::new(1, 0));
        assert_eq!(line[2], HexCoord::new(2, 0));
    }

    #[test]
    fn test_line_of_sight() {
        let start = HexCoord::origin();
        let end = HexCoord::new(3, 0);
        
        // Clear line of sight
        assert!(start.has_line_of_sight(end, &[]));
        
        // Blocked line of sight
        let blocked = vec![HexCoord::new(1, 0)];
        assert!(!start.has_line_of_sight(end, &blocked));
        
        // Blocking start or end doesn't block line of sight
        let blocked_start = vec![HexCoord::origin()];
        assert!(start.has_line_of_sight(end, &blocked_start));
        
        let blocked_end = vec![HexCoord::new(3, 0)];
        assert!(start.has_line_of_sight(end, &blocked_end));
    }
}