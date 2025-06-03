use bluefelt_core::{bundle::BundleMap, engine::{load_initial_state, apply_verb}};
use serde_json::{json, Value};

#[test]
fn test_three_mens_morris_action_map() {
    // Load bundles from directory
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("three-mens-morris").expect("Failed to get bundle");
    
    let mut state = load_initial_state(&bundle);
    
    // Place all pieces to get to movement phase
    let placement_moves = vec![
        ("/zones/board/cells/0/0", "piece_p1"),
        ("/zones/board/cells/0/2", "piece_p2"), 
        ("/zones/board/cells/1/1", "piece_p1"),
        ("/zones/board/cells/2/0", "piece_p2"),
        ("/zones/board/cells/2/2", "piece_p1"),
        ("/zones/board/cells/0/1", "piece_p2"),
    ];
    
    for (location, entity) in placement_moves {
        let place_args = json!({"location": location, "entity": entity});
        apply_verb(&mut state, "place", &place_args, &bundle).unwrap();
        apply_verb(&mut state, "nextTurn", &json!({}), &bundle).unwrap();
    }
    
    // Transition to movement phase
    let set_phase_args = json!({"phaseSet": "game", "phase": "movement"});
    apply_verb(&mut state, "setPhase", &set_phase_args, &bundle).unwrap();
    
    println!("=== Game State in Movement Phase ===");
    println!("Current player: {:?}", state.get("currentPlayer"));
    println!("Current phase: {:?}", state.get("phases"));
    
    // Check what UI/actionMap looks like in movement phase
    let ui = state.get("ui");
    println!("UI state: {:?}", ui);
    
    if let Some(ui_obj) = ui {
        if let Some(action_map) = ui_obj.get("actionMap") {
            println!("Action map: {:?}", action_map);
            
            // Check specifically for player actions
            if let Some(p1_actions) = action_map.get("p1") {
                println!("P1 actions in movement phase: {:?}", p1_actions);
            }
            if let Some(p2_actions) = action_map.get("p2") {
                println!("P2 actions in movement phase: {:?}", p2_actions);
            }
        } else {
            println!("No action map found in UI");
        }
    } else {
        println!("No UI state found");
    }
    
    // Check board state
    let board = state.get("zones").and_then(|z| z.get("board"));
    println!("Board state: {:?}", board);
}