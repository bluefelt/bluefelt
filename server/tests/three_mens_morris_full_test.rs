use bluefelt_core::{bundle::BundleMap, engine::{load_initial_state, apply_verb}};
use serde_json::{json, Value};

#[test]
fn test_three_mens_morris_full_game() {
    // Load bundles from directory
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get bundle");
    
    let mut state = load_initial_state(&bundle);
    
    println!("=== Initial Game State ===");
    println!("Current player: {:?}", state.get("currentPlayer"));
    println!("Current phase: {:?}", state.get("phases"));
    println!("Board: {:?}", state.get("zones").and_then(|z| z.get("board")));
    
    // Test placement phase - place 3 pieces for each player
    let placement_moves = vec![
        ("/zones/board/cells/0/0", "p1", "piece_p1"),  // P1 places
        ("/zones/board/cells/0/2", "p2", "piece_p2"),  // P2 places
        ("/zones/board/cells/1/1", "p1", "piece_p1"),  // P1 places
        ("/zones/board/cells/2/0", "p2", "piece_p2"),  // P2 places
        ("/zones/board/cells/2/2", "p1", "piece_p1"),  // P1 places
        ("/zones/board/cells/0/1", "p2", "piece_p2"),  // P2 places
    ];
    
    for (i, (location, player, entity)) in placement_moves.iter().enumerate() {
        println!("\n=== Placement Move {} ===", i + 1);
        
        let current_player = state.get("currentPlayer").and_then(|p| p.as_str()).unwrap_or("");
        println!("Expected player: {}, Current player: {}", player, current_player);
        
        // Check if current player matches expected
        assert_eq!(current_player, *player, "Wrong player turn at move {}", i + 1);
        
        // Apply placement verb directly
        let place_args = json!({
            "location": location,
            "entity": entity
        });
        
        let result = apply_verb(&mut state, "place", &place_args, &bundle);
        println!("Place result: {:?}", result);
        assert!(result.is_ok(), "Failed placement at move {}: {:?}", i + 1, result);
        
        // Apply nextTurn verb
        let next_turn_result = apply_verb(&mut state, "nextTurn", &json!({}), &bundle);
        println!("Next turn result: {:?}", next_turn_result);
        assert!(next_turn_result.is_ok(), "Failed to advance turn");
        
        // Check board state after move
        let board = state.get("zones").and_then(|z| z.get("board")).unwrap();
        println!("Board after move {}: {:?}", i + 1, board);
        
        // Check phase
        let phases = state.get("phases").unwrap();
        println!("Phases after move {}: {:?}", i + 1, phases);
        
        // Count pieces for each player
        let piece_count_p1 = count_pieces_on_board(&state, "piece_p1");
        let piece_count_p2 = count_pieces_on_board(&state, "piece_p2");
        println!("Piece counts - P1: {}, P2: {}", piece_count_p1, piece_count_p2);
        
        // After 6th move, check if we should transition to movement phase
        if i == 5 {  // After all 6 placements
            println!("\n=== Checking Phase Transition ===");
            
            // Apply setPhase verb manually to test transition
            let set_phase_args = json!({
                "phaseSet": "game",
                "phase": "movement"
            });
            
            // Check if conditions are met for phase transition
            if piece_count_p1 == 3 && piece_count_p2 == 3 {
                println!("✓ Both players have 3 pieces, attempting phase transition");
                let phase_result = apply_verb(&mut state, "setPhase", &set_phase_args, &bundle);
                println!("Phase transition result: {:?}", phase_result);
                
                if phase_result.is_ok() {
                    let new_phases = state.get("phases").unwrap();
                    println!("New phases after transition: {:?}", new_phases);
                }
            }
        }
    }
    
    println!("\n=== After All Placements ===");
    let phases = state.get("phases").unwrap();
    println!("Final phases: {:?}", phases);
    
    // Check if we're in movement phase
    let game_phase = phases.get("game").and_then(|p| p.as_str());
    println!("Game phase: {:?}", game_phase);
    
    if game_phase == Some("movement") {
        println!("✓ Successfully transitioned to movement phase");
        
        // Test movement phase actions
        println!("\n=== Testing Movement Phase ===");
        
        let current_player = state.get("currentPlayer").and_then(|p| p.as_str()).unwrap().to_string();
        println!("Current player in movement phase: {}", current_player);
        
        // Try to select a piece
        println!("\n--- Testing piece selection ---");
        let select_args = json!({
            "location": "/zones/board/cells/0/0",
            "player": current_player
        });
        
        let result = apply_verb(&mut state, "selectEntity", &select_args, &bundle);
        println!("Select piece result: {:?}", result);
        
        if result.is_ok() {
            // Check selection state
            let selection = state.get("selection");
            println!("Selection state: {:?}", selection);
            
            // Try to move the selected piece
            println!("\n--- Testing piece movement ---");
            let move_args = json!({
                "target": "/zones/board/cells/1/0",
                "player": current_player
            });
            
            let result = apply_verb(&mut state, "moveSelected", &move_args, &bundle);
            println!("Move piece result: {:?}", result);
            
            if result.is_ok() {
                let board = state.get("zones").and_then(|z| z.get("board"));
                println!("Board after movement: {:?}", board);
                println!("✓ Movement successful");
            } else {
                println!("✗ Movement failed: {:?}", result);
            }
        } else {
            println!("✗ Piece selection failed: {:?}", result);
        }
    } else {
        println!("✗ Failed to transition to movement phase, still in: {:?}", game_phase);
        
        // Let's manually check why the transition didn't happen
        println!("\n=== Debugging Phase Transition ===");
        let piece_count_p1 = count_pieces_on_board(&state, "piece_p1");
        let piece_count_p2 = count_pieces_on_board(&state, "piece_p2");
        println!("Final piece counts - P1: {}, P2: {}", piece_count_p1, piece_count_p2);
        
        // Try manual phase transition
        let set_phase_args = json!({
            "phaseSet": "game",
            "phase": "movement"
        });
        let phase_result = apply_verb(&mut state, "setPhase", &set_phase_args, &bundle);
        println!("Manual phase transition result: {:?}", phase_result);
        
        if phase_result.is_ok() {
            let new_phases = state.get("phases").unwrap();
            println!("Phases after manual transition: {:?}", new_phases);
        }
    }
}

fn count_pieces_on_board(state: &Value, piece_type: &str) -> usize {
    let board = state.get("zones").and_then(|z| z.get("board"));
    if let Some(board) = board {
        if let Some(cells) = board.get("cells").and_then(|c| c.as_array()) {
            let mut count = 0;
            for row in cells {
                if let Some(row_array) = row.as_array() {
                    for cell in row_array {
                        if let Some(entity) = cell.get("entity").and_then(|e| e.as_str()) {
                            if entity == piece_type {
                                count += 1;
                            }
                        }
                    }
                }
            }
            return count;
        }
    }
    0
}