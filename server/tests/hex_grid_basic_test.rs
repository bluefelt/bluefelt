// Basic Hex Grid Functionality Test
// Tests core hex grid zone creation and validation

use bluefelt_core::engine::state::load_initial_state_with_player_names;
use bluefelt_core::bundle::Bundle;
use serde_json::json;
use rand::SeedableRng;
use rand::rngs::StdRng;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_basic_hex_bundle() -> Bundle {
        let manifest = json!({
            "gameId": "hex-test",
            "version": "1.0",
            "specVersion": "1.0",
            "metadata": {
                "name": "hex-test",
                "author": "test",
                "description": "test hex grid game",
                "players": {
                    "min": 2,
                    "max": 2
                }
            }
        });

        let zones = json!([
            {
                "id": "hex_board",
                "type": "hexgrid",
                "shapeMeta": {
                    "layout": "flat",
                    "size": 2
                },
                "contents": "empty"
            },
            {
                "id": "rect_hex_board", 
                "type": "hexgrid",
                "shapeMeta": {
                    "layout": "pointy",
                    "rows": 3,
                    "cols": 4
                },
                "contents": "empty"
            }
        ]);

        let phases = json!([
            {
                "id": "main",
                "phases": [
                    {
                        "id": "play",
                        "initial": true
                    }
                ]
            }
        ]);

        Bundle {
            game_id: "hex-test".to_string(),
            manifest: serde_json::from_value(manifest).unwrap(),
            zones: zones,
            phases: phases,
            entities: json!({}),
            actions: json!([]),
        }
    }

    #[test]
    fn test_hexgrid_zone_creation() {
        let bundle = create_basic_hex_bundle();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut rng = StdRng::seed_from_u64(12345);

        let state = load_initial_state_with_player_names(&bundle, &player_names, &mut rng);

        // Check that hex zones were created
        assert!(state["zones"]["hex_board"].is_object());
        assert_eq!(state["zones"]["hex_board"]["type"], "hexgrid");
        assert_eq!(state["zones"]["hex_board"]["layout"], "flat");
        
        // Check that cells were created as a map
        let cells = &state["zones"]["hex_board"]["cells"];
        assert!(cells.is_object());
        
        // For a hex grid with radius 2, we should have specific number of cells
        // Radius 2 hex grid should have 1 + 6 + 12 = 19 cells
        let cells_map = cells.as_object().unwrap();
        assert_eq!(cells_map.len(), 19);

        // Check that specific coordinates exist
        assert!(cells_map.contains_key("0,0")); // Center
        assert!(cells_map.contains_key("1,0")); // East neighbor
        assert!(cells_map.contains_key("0,1")); // Southeast neighbor
        assert!(cells_map.contains_key("-1,1")); // Southwest neighbor
        assert!(cells_map.contains_key("-1,0")); // West neighbor
        assert!(cells_map.contains_key("0,-1")); // Northwest neighbor
        assert!(cells_map.contains_key("1,-1")); // Northeast neighbor

        // Check rectangular hex grid
        assert!(state["zones"]["rect_hex_board"].is_object());
        assert_eq!(state["zones"]["rect_hex_board"]["type"], "hexgrid");
        assert_eq!(state["zones"]["rect_hex_board"]["layout"], "pointy");
        
        let rect_cells = &state["zones"]["rect_hex_board"]["cells"];
        assert!(rect_cells.is_object());
        
        // 3 rows x 4 cols = 12 cells
        let rect_cells_map = rect_cells.as_object().unwrap();
        assert_eq!(rect_cells_map.len(), 12);
    }

    #[test]
    fn test_hex_coordinate_format() {
        let bundle = create_basic_hex_bundle();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut rng = StdRng::seed_from_u64(12345);

        let state = load_initial_state_with_player_names(&bundle, &player_names, &mut rng);

        let cells = state["zones"]["hex_board"]["cells"].as_object().unwrap();
        
        // Check that all cell keys are in "q,r" format
        for key in cells.keys() {
            let parts: Vec<&str> = key.split(',').collect();
            assert_eq!(parts.len(), 2, "Cell key should be in 'q,r' format: {}", key);
            
            // Check that both parts are valid integers
            parts[0].parse::<i32>().expect(&format!("First part should be integer: {}", parts[0]));
            parts[1].parse::<i32>().expect(&format!("Second part should be integer: {}", parts[1]));
        }
    }

    #[test]
    fn test_hex_cells_initially_empty() {
        let bundle = create_basic_hex_bundle();
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut rng = StdRng::seed_from_u64(12345);

        let state = load_initial_state_with_player_names(&bundle, &player_names, &mut rng);

        let cells = state["zones"]["hex_board"]["cells"].as_object().unwrap();
        
        // All cells should initially be null (empty)
        for (key, value) in cells {
            assert!(value.is_null(), "Cell {} should be initially empty, got: {:?}", key, value);
        }
    }

    #[test]
    fn test_hex_grid_with_initial_contents() {
        let mut bundle = create_basic_hex_bundle();
        
        // Modify the bundle to have initial contents
        bundle.zones = json!([
            {
                "id": "hex_board_with_contents",
                "type": "hexgrid", 
                "shapeMeta": {
                    "layout": "flat",
                    "size": 1
                },
                "contents": {"entity": "test_piece"}
            }
        ]);

        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        let mut rng = StdRng::seed_from_u64(12345);

        let state = load_initial_state_with_player_names(&bundle, &player_names, &mut rng);

        let cells = state["zones"]["hex_board_with_contents"]["cells"].as_object().unwrap();
        
        // Size 1 hex grid should have 7 cells (1 center + 6 neighbors)
        assert_eq!(cells.len(), 7);
        
        // All cells should contain the initial entity
        for (key, value) in cells {
            assert_eq!(value["entity"], "test_piece", "Cell {} should contain test_piece", key);
        }
    }
}