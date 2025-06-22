// Hex Distance Calculations
// Manhattan distance and related operations for hex grids

use super::HexCoord;

impl HexCoord {
    /// Calculate Manhattan distance between two hex coordinates
    /// Uses cube coordinate system for accurate distance calculation
    pub fn distance(self, other: HexCoord) -> i32 {
        let cube1 = self.to_cube();
        let cube2 = other.to_cube();
        
        ((cube1.x - cube2.x).abs() + (cube1.y - cube2.y).abs() + (cube1.z - cube2.z).abs()) / 2
    }

    /// Calculate distance from origin (0, 0)
    pub fn distance_from_origin(self) -> i32 {
        self.distance(HexCoord::origin())
    }

    /// Find the shortest path between two hexes using A* algorithm
    /// Returns None if no path exists (for future obstacle support)
    pub fn find_path(self, target: HexCoord) -> Option<Vec<HexCoord>> {
        // For now, simple direct line (will be enhanced for pathfinding with obstacles)
        Some(self.line_to(target))
    }

    /// Find all hexes reachable within a movement cost
    /// Basic version - will be enhanced for terrain costs later
    pub fn reachable_hexes(self, movement: i32) -> Vec<HexCoord> {
        self.hexes_within_range(movement)
    }

    /// Get hexes sorted by distance from this hex
    pub fn sorted_by_distance(self, hexes: &[HexCoord]) -> Vec<(HexCoord, i32)> {
        let mut with_distances: Vec<(HexCoord, i32)> = hexes
            .iter()
            .map(|&hex| (hex, self.distance(hex)))
            .collect();
        
        with_distances.sort_by_key(|(_, distance)| *distance);
        with_distances
    }

    /// Find the closest hex from a list of candidates
    pub fn closest_hex(self, candidates: &[HexCoord]) -> Option<HexCoord> {
        candidates
            .iter()
            .min_by_key(|&&hex| self.distance(hex))
            .copied()
    }

    /// Check if a hex is within a certain distance
    pub fn is_within_distance(self, other: HexCoord, max_distance: i32) -> bool {
        self.distance(other) <= max_distance
    }

    /// Get all hexes at exactly a certain distance
    pub fn hexes_at_distance(self, distance: i32) -> Vec<HexCoord> {
        if distance == 0 {
            return vec![self];
        }
        
        self.ring(distance)
    }
}

/// Helper functions for distance calculations
pub mod distance_utils {
    use super::*;

    /// Calculate the centroid (average position) of a group of hexes
    pub fn centroid(hexes: &[HexCoord]) -> Option<HexCoord> {
        if hexes.is_empty() {
            return None;
        }

        let sum_q: i32 = hexes.iter().map(|h| h.q).sum();
        let sum_r: i32 = hexes.iter().map(|h| h.r).sum();
        let len = hexes.len() as i32;

        Some(HexCoord::new(sum_q / len, sum_r / len))
    }

    /// Find the hex that minimizes total distance to all other hexes
    pub fn median_hex(hexes: &[HexCoord]) -> Option<HexCoord> {
        if hexes.is_empty() {
            return None;
        }

        hexes
            .iter()
            .min_by_key(|&&center| {
                hexes.iter().map(|&h| center.distance(h)).sum::<i32>()
            })
            .copied()
    }

    /// Calculate the diameter (maximum distance between any two hexes)
    pub fn diameter(hexes: &[HexCoord]) -> i32 {
        let mut max_distance = 0;
        
        for (i, &hex1) in hexes.iter().enumerate() {
            for &hex2 in hexes.iter().skip(i + 1) {
                max_distance = max_distance.max(hex1.distance(hex2));
            }
        }
        
        max_distance
    }

    /// Find all pairs of hexes within a certain distance
    pub fn pairs_within_distance(hexes: &[HexCoord], max_distance: i32) -> Vec<(HexCoord, HexCoord)> {
        let mut pairs = Vec::new();
        
        for (i, &hex1) in hexes.iter().enumerate() {
            for &hex2 in hexes.iter().skip(i + 1) {
                if hex1.distance(hex2) <= max_distance {
                    pairs.push((hex1, hex2));
                }
            }
        }
        
        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::distance_utils::*;

    #[test]
    fn test_hex_distance() {
        let origin = HexCoord::origin();
        
        // Distance to self should be 0
        assert_eq!(origin.distance(origin), 0);
        
        // Distance to neighbors should be 1
        for neighbor in origin.neighbors() {
            assert_eq!(origin.distance(neighbor), 1);
        }
        
        // Test specific distances
        assert_eq!(origin.distance(HexCoord::new(2, 0)), 2);
        assert_eq!(origin.distance(HexCoord::new(1, 1)), 2);
        assert_eq!(origin.distance(HexCoord::new(2, 1)), 3);
        assert_eq!(origin.distance(HexCoord::new(-1, 2)), 2);
    }

    #[test]
    fn test_distance_symmetry() {
        let hex1 = HexCoord::new(1, 2);
        let hex2 = HexCoord::new(-2, 1);
        
        assert_eq!(hex1.distance(hex2), hex2.distance(hex1));
    }

    #[test]
    fn test_distance_from_origin() {
        let hex = HexCoord::new(3, -1);
        assert_eq!(hex.distance_from_origin(), hex.distance(HexCoord::origin()));
    }

    #[test]
    fn test_reachable_hexes() {
        let origin = HexCoord::origin();
        
        let reachable_1 = origin.reachable_hexes(1);
        assert_eq!(reachable_1.len(), 7); // origin + 6 neighbors
        
        let reachable_2 = origin.reachable_hexes(2);
        assert_eq!(reachable_2.len(), 19); // 1 + 6 + 12
    }

    #[test]
    fn test_sorted_by_distance() {
        let origin = HexCoord::origin();
        let hexes = vec![
            HexCoord::new(3, 0),   // distance 3
            HexCoord::new(1, 0),   // distance 1
            HexCoord::new(2, 1),   // distance 3
            HexCoord::new(0, 1),   // distance 1
        ];
        
        let sorted = origin.sorted_by_distance(&hexes);
        
        assert_eq!(sorted[0].1, 1); // closest
        assert_eq!(sorted[1].1, 1);
        assert_eq!(sorted[2].1, 3); // farthest
        assert_eq!(sorted[3].1, 3);
    }

    #[test]
    fn test_closest_hex() {
        let origin = HexCoord::origin();
        let candidates = vec![
            HexCoord::new(5, 0),
            HexCoord::new(1, 0),
            HexCoord::new(3, 2),
        ];
        
        let closest = origin.closest_hex(&candidates).unwrap();
        assert_eq!(closest, HexCoord::new(1, 0));
    }

    #[test]
    fn test_is_within_distance() {
        let origin = HexCoord::origin();
        
        assert!(origin.is_within_distance(HexCoord::new(1, 0), 2));
        assert!(origin.is_within_distance(HexCoord::new(2, 0), 2));
        assert!(!origin.is_within_distance(HexCoord::new(3, 0), 2));
    }

    #[test]
    fn test_hexes_at_distance() {
        let origin = HexCoord::origin();
        
        let at_distance_0 = origin.hexes_at_distance(0);
        assert_eq!(at_distance_0.len(), 1);
        assert_eq!(at_distance_0[0], origin);
        
        let at_distance_1 = origin.hexes_at_distance(1);
        assert_eq!(at_distance_1.len(), 6);
        
        let at_distance_2 = origin.hexes_at_distance(2);
        assert_eq!(at_distance_2.len(), 12);
        
        // Verify all hexes are at correct distance
        for hex in at_distance_2 {
            assert_eq!(origin.distance(hex), 2);
        }
    }

    #[test]
    fn test_centroid() {
        let hexes = vec![
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            HexCoord::new(1, 1),
        ];
        
        let centroid = centroid(&hexes).unwrap();
        assert_eq!(centroid, HexCoord::new(1, 0)); // (0+2+1)/3, (0+0+1)/3
    }

    #[test]
    fn test_median_hex() {
        let hexes = vec![
            HexCoord::new(0, 0),
            HexCoord::new(2, 0),
            HexCoord::new(4, 0),
        ];
        
        let median = median_hex(&hexes).unwrap();
        assert_eq!(median, HexCoord::new(2, 0)); // Minimizes total distance
    }

    #[test]
    fn test_diameter() {
        let hexes = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(3, 1),
        ];
        
        let diam = diameter(&hexes);
        assert_eq!(diam, 4); // Distance from (0,0) to (3,1)
    }

    #[test]
    fn test_pairs_within_distance() {
        let hexes = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(3, 0),
            HexCoord::new(0, 1),
        ];
        
        let pairs = pairs_within_distance(&hexes, 1);
        assert_eq!(pairs.len(), 3); // (0,0)-(1,0), (0,0)-(0,1), and (1,0)-(0,1)
        
        let pairs = pairs_within_distance(&hexes, 2);
        assert_eq!(pairs.len(), 4); // (0,0)-(1,0), (0,0)-(0,1), (1,0)-(0,1), (1,0)-(3,0)
    }

    #[test]
    fn test_empty_collections() {
        assert!(centroid(&[]).is_none());
        assert!(median_hex(&[]).is_none());
        assert_eq!(diameter(&[]), 0);
        assert_eq!(pairs_within_distance(&[], 5).len(), 0);
    }
}