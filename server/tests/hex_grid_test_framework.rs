// Hex Grid Test Framework
// Comprehensive testing utilities for hex grid functionality

use serde_json::{json, Value};
use std::collections::HashMap;

#[cfg(test)]
mod hex_grid_tests {
    use super::*;

    /// Test framework for hex grid functionality
    pub struct HexGridTestFramework {
        state: Value,
        game_config: Value,
    }

    impl HexGridTestFramework {
        pub fn new_basic_hex_board(size: usize) -> Self {
            let state = json!({
                "zones": {
                    "hex_board": {
                        "shape": "hexgrid",
                        "shapeMeta": {
                            "layout": "flat",
                            "size": size,
                            "coordinates": "axial"
                        },
                        "cells": Self::generate_hex_cells(size)
                    }
                },
                "players": ["p1", "p2"],
                "currentPlayer": "p1",
                "gameStatus": {
                    "state": "playing",
                    "winner": null
                }
            });

            Self {
                state,
                game_config: json!({}),
            }
        }

        fn generate_hex_cells(size: usize) -> Value {
            // Generate empty hex grid in axial coordinates
            let mut cells = HashMap::new();
            
            for q in -(size as i32)..=(size as i32) {
                for r in -(size as i32)..=(size as i32) {
                    if q.abs() + r.abs() + (q + r).abs() <= 2 * (size as i32) {
                        let coord = format!("{}_{}", q, r);
                        cells.insert(coord, json!(null));
                    }
                }
            }
            
            json!(cells)
        }

        pub fn place_entity(&mut self, location: &str, entity: &str) -> Result<(), String> {
            // Simulate placing entity at hex location
            let cells = self.state["zones"]["hex_board"]["cells"]
                .as_object_mut()
                .ok_or("Invalid cells structure")?;
            
            cells.insert(location.to_string(), json!({"entity": entity}));
            Ok(())
        }

        pub fn test_hex_distance(&self, loc1: &str, loc2: &str, expected: i32) -> bool {
            // Parse axial coordinates from location strings
            let (q1, r1) = self.parse_axial_location(loc1).unwrap();
            let (q2, r2) = self.parse_axial_location(loc2).unwrap();
            
            // Calculate hex distance using cube coordinates
            let distance = ((q1 - q2).abs() + (q1 + r1 - q2 - r2).abs() + (r1 - r2).abs()) / 2;
            distance == expected
        }

        fn parse_axial_location(&self, location: &str) -> Result<(i32, i32), String> {
            let parts: Vec<&str> = location.split('_').collect();
            if parts.len() != 2 {
                return Err("Invalid location format".to_string());
            }
            
            let q = parts[0].parse::<i32>().map_err(|_| "Invalid q coordinate")?;
            let r = parts[1].parse::<i32>().map_err(|_| "Invalid r coordinate")?;
            
            Ok((q, r))
        }

        pub fn test_hex_neighbors(&self, location: &str) -> Vec<String> {
            let (q, r) = self.parse_axial_location(location).unwrap();
            
            // Six directions in axial coordinates
            let directions = [
                (1, 0),   // East
                (0, 1),   // Southeast  
                (-1, 1),  // Southwest
                (-1, 0),  // West
                (0, -1),  // Northwest
                (1, -1),  // Northeast
            ];
            
            directions
                .iter()
                .map(|(dq, dr)| format!("{}_{}", q + dq, r + dr))
                .collect()
        }

        pub fn test_line_of_hexes(&self, start: &str, end: &str) -> Vec<String> {
            let (q1, r1) = self.parse_axial_location(start).unwrap();
            let (q2, r2) = self.parse_axial_location(end).unwrap();
            
            let distance = self.test_hex_distance(start, end, 0); // Get actual distance
            let mut line = Vec::new();
            
            // Linear interpolation in cube coordinates
            for i in 0..=((q1-q2).abs().max((r1-r2).abs()).max((-q1-r1+q2+r2).abs())) {
                let t = i as f32 / ((q1-q2).abs().max((r1-r2).abs()).max((-q1-r1+q2+r2).abs())) as f32;
                let q = ((1.0 - t) * q1 as f32 + t * q2 as f32).round() as i32;
                let r = ((1.0 - t) * r1 as f32 + t * r2 as f32).round() as i32;
                line.push(format!("{}_{}", q, r));
            }
            
            line
        }

        pub fn test_connected_region(&self, start: &str, entity_type: &str) -> Vec<String> {
            let mut visited = std::collections::HashSet::new();
            let mut region = Vec::new();
            let mut stack = vec![start.to_string()];
            
            while let Some(current) = stack.pop() {
                if visited.contains(&current) {
                    continue;
                }
                
                visited.insert(current.clone());
                
                // Check if current location has the specified entity
                if let Some(entity) = self.get_entity_at(&current) {
                    if entity == entity_type {
                        region.push(current.clone());
                        
                        // Add neighbors to stack
                        for neighbor in self.test_hex_neighbors(&current) {
                            if !visited.contains(&neighbor) {
                                stack.push(neighbor);
                            }
                        }
                    }
                }
            }
            
            region
        }

        fn get_entity_at(&self, location: &str) -> Option<String> {
            self.state["zones"]["hex_board"]["cells"][location]["entity"]
                .as_str()
                .map(|s| s.to_string())
        }
    }

    // Phase 1 Tests: Basic Hex Grid Math
    #[test]
    fn test_phase1_hex_coordinates() {
        let framework = HexGridTestFramework::new_basic_hex_board(2);
        
        // Test distance calculations
        assert!(framework.test_hex_distance("0_0", "1_0", 1));
        assert!(framework.test_hex_distance("0_0", "2_0", 2));
        assert!(framework.test_hex_distance("0_0", "1_1", 2));
        assert!(framework.test_hex_distance("-1_1", "1_-1", 3));
    }

    #[test]
    fn test_phase1_hex_neighbors() {
        let framework = HexGridTestFramework::new_basic_hex_board(2);
        
        let neighbors = framework.test_hex_neighbors("0_0");
        assert_eq!(neighbors.len(), 6);
        assert!(neighbors.contains(&"1_0".to_string()));
        assert!(neighbors.contains(&"0_1".to_string()));
        assert!(neighbors.contains(&"-1_1".to_string()));
        assert!(neighbors.contains(&"-1_0".to_string()));
        assert!(neighbors.contains(&"0_-1".to_string()));
        assert!(neighbors.contains(&"1_-1".to_string()));
    }

    // Phase 2 Tests: Hex Interactions
    #[test]
    fn test_phase2_adjacency_placement() {
        let mut framework = HexGridTestFramework::new_basic_hex_board(3);
        
        // Place initial entity
        framework.place_entity("0_0", "territory_p1").unwrap();
        
        // Test that neighbors are adjacent
        let neighbors = framework.test_hex_neighbors("0_0");
        for neighbor in neighbors {
            // In real implementation, would test adjacency condition
            // For now, just verify neighbor calculation works
            assert!(framework.test_hex_distance("0_0", &neighbor, 1));
        }
    }

    #[test]
    fn test_phase2_hex_line_drawing() {
        let framework = HexGridTestFramework::new_basic_hex_board(5);
        
        let line = framework.test_line_of_hexes("0_0", "3_0");
        assert!(line.len() >= 4); // Should include start, end, and intermediate hexes
        assert_eq!(line.first().unwrap(), "0_0");
        assert_eq!(line.last().unwrap(), "3_0");
    }

    // Phase 3 Tests: Territory Control
    #[test]
    fn test_phase3_connected_regions() {
        let mut framework = HexGridTestFramework::new_basic_hex_board(3);
        
        // Create connected region
        framework.place_entity("0_0", "territory_p1").unwrap();
        framework.place_entity("1_0", "territory_p1").unwrap();
        framework.place_entity("0_1", "territory_p1").unwrap();
        
        // Create separate region
        framework.place_entity("2_2", "territory_p1").unwrap();
        
        let region = framework.test_connected_region("0_0", "territory_p1");
        assert_eq!(region.len(), 3); // Should find connected group of 3
        
        let separate_region = framework.test_connected_region("2_2", "territory_p1");
        assert_eq!(separate_region.len(), 1); // Isolated hex
    }

    // Integration test for complete hex game flow
    #[test]
    fn test_hex_tic_tac_toe_flow() {
        let mut framework = HexGridTestFramework::new_basic_hex_board(2);
        
        // Simulate a game of hex tic-tac-toe
        framework.place_entity("0_0", "mark_p1").unwrap();
        framework.place_entity("1_0", "mark_p2").unwrap();
        framework.place_entity("0_1", "mark_p1").unwrap();
        framework.place_entity("-1_1", "mark_p2").unwrap();
        framework.place_entity("-1_0", "mark_p1").unwrap(); // Winning move
        
        // In real implementation, would test win detection
        // For now, verify placement worked
        assert_eq!(framework.get_entity_at("0_0"), Some("mark_p1".to_string()));
        assert_eq!(framework.get_entity_at("-1_0"), Some("mark_p1".to_string()));
    }

    // Performance test for larger hex grids
    #[test]
    fn test_hex_grid_performance() {
        use std::time::Instant;
        
        let framework = HexGridTestFramework::new_basic_hex_board(50);
        
        // Test neighbor calculation performance
        let start = Instant::now();
        for _ in 0..1000 {
            let _neighbors = framework.test_hex_neighbors("0_0");
        }
        assert!(start.elapsed().as_millis() < 100); // Should be very fast
        
        // Test distance calculation performance
        let start = Instant::now();
        for i in 0..100 {
            let _distance = framework.test_hex_distance("0_0", &format!("{}_{}", i % 10, i % 10), 0);
        }
        assert!(start.elapsed().as_millis() < 50);
    }
}

// Benchmarking utilities for hex operations
#[cfg(test)]
mod hex_benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_hex_distance_calculation() {
        let iterations = 10000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let q1 = (i % 100) as i32;
            let r1 = ((i * 2) % 100) as i32;
            let q2 = ((i * 3) % 100) as i32;
            let r2 = ((i * 4) % 100) as i32;
            
            // Hex distance calculation
            let _distance = ((q1 - q2).abs() + (q1 + r1 - q2 - r2).abs() + (r1 - r2).abs()) / 2i32;
        }
        
        let elapsed = start.elapsed();
        println!("Hex distance calculation: {:.2}μs per operation", 
                elapsed.as_nanos() as f64 / iterations as f64 / 1000.0);
        
        // Should be under 1μs per operation
        assert!(elapsed.as_nanos() / iterations < 1000);
    }

    #[test] 
    fn benchmark_hex_neighbor_generation() {
        let iterations = 10000;
        let start = Instant::now();
        
        for i in 0..iterations {
            let q = (i % 100) as i32;
            let r = ((i * 2) % 100) as i32;
            
            // Generate 6 neighbors
            let _neighbors: Vec<(i32, i32)> = [
                (1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)
            ].iter().map(|(dq, dr)| (q + dq, r + dr)).collect();
        }
        
        let elapsed = start.elapsed();
        println!("Hex neighbor generation: {:.2}μs per operation",
                elapsed.as_nanos() as f64 / iterations as f64 / 1000.0);
    }
}