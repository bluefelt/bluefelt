use serde_json::json;
use bluefelt_core::engine::state::load_initial_state;
use bluefelt_core::engine::verbs::multi_step::{
    parse_multi_step_action, execute_multi_step_finalize, MultiStepState, MultiStepResponse
};
use bluefelt_core::bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange};
use std::collections::HashMap;

#[test]
fn test_multi_step_move_generates_board_patches() {
    // Create a simple test bundle with a move action
    let actions = vec![
        json!({
            "id": "movePiece",
            "isMultiStep": true,
            "stateStore": ["selectedPiece", "destination"],
            "steps": [
                {
                    "id": "selectPiece",
                    "as": "bf.selectEntity",
                    "with": {"zone": "/zones/board"},
                    "store": "selectedPiece"
                },
                {
                    "id": "selectDestination",
                    "as": "bf.selectLocation",
                    "with": {"zone": "/zones/board"},
                    "store": "destination"
                }
            ],
            "result": {
                "as": "moveEntity",
                "with": {
                    "from": "{selectedPiece}",
                    "to": "{destination}"
                }
            }
        })
    ];
    
    let manifest = Manifest {
        game_id: "test-game".to_string(),
        version: "1.0".to_string(),
        spec_version: "1.0".to_string(),
        metadata: ManifestMetadata {
            name: "Test Game".to_string(),
            author: "Test Author".to_string(),
            description: "Test game for multi-step actions".to_string(),
            players: PlayersRange {
                min: 2,
                max: 2,
            },
        },
        phases: None,
        setup: None,
        zone_groups: None,
    };
    
    let bundle = Bundle {
        game_id: "test-game".to_string(),
        manifest,
        actions: json!(actions),
        entities: json!([]),
        phases: json!([]),
        zones: json!([{
            "id": "board",
            "type": "grid",
            "dimensions": {
                "rows": 3,
                "cols": 3
            }
        }]),
    };
    
    // Create initial state with a piece on the board
    let mut state = load_initial_state(&bundle);
    state["zones"]["board"]["cells"][0][0] = json!({"entity": "piece_p1"});
    state["currentPlayer"] = json!("p1");
    
    // Parse the multi-step action
    let actions_array = bundle.actions.as_array().unwrap();
    let action_def = parse_multi_step_action(&actions_array[0]).unwrap();
    
    // Create multi-step state and simulate selections
    let mut multi_step_state = MultiStepState {
        action_id: "movePiece".to_string(),
        current_step: 0,
        stored_values: HashMap::new(),
        can_cancel: true,
        deferred_logs: vec![],
        created_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
    };
    
    // Simulate selecting the piece at 0,0
    multi_step_state.stored_values.insert(
        "selectedPiece".to_string(), 
        json!("/zones/board/cells/0/0")
    );
    multi_step_state.current_step = 1;
    
    // Simulate selecting destination at 1,1
    multi_step_state.stored_values.insert(
        "destination".to_string(),
        json!("/zones/board/cells/1/1")
    );
    multi_step_state.current_step = 2;
    
    // Execute the finalize step
    let result = execute_multi_step_finalize(
        &bundle,
        &mut state,
        "p1",
        &multi_step_state,
        &action_def
    );
    
    // Verify the result
    match result {
        Ok(MultiStepResponse::Completed { patches, .. }) => {
            // Should have 2 patches: one to remove from source, one to add to destination
            assert_eq!(patches.len(), 2, "Expected 2 patches for move operation");
            
            // Verify the patches have correct paths (not formatted coordinates)
            let patch0_path = patches[0]["path"].as_str().unwrap();
            let patch1_path = patches[1]["path"].as_str().unwrap();
            
            assert!(
                patch0_path == "/zones/board/cells/0/0" || patch0_path == "/zones/board/cells/1/1",
                "Patch path should be a zone path, not formatted coordinates. Got: {}",
                patch0_path
            );
            assert!(
                patch1_path == "/zones/board/cells/0/0" || patch1_path == "/zones/board/cells/1/1",
                "Patch path should be a zone path, not formatted coordinates. Got: {}",
                patch1_path
            );
            
            // Verify one patch removes and one adds
            let has_remove = patches.iter().any(|p| p["value"].is_null());
            let has_add = patches.iter().any(|p| !p["value"].is_null());
            
            assert!(has_remove, "Should have a patch that removes the piece");
            assert!(has_add, "Should have a patch that adds the piece");
        }
        _ => panic!("Expected Completed response, got: {:?}", result)
    }
    
    // Verify the state was actually updated
    assert!(state["zones"]["board"]["cells"][0][0].is_null(), "Source cell should be empty");
    assert_eq!(
        state["zones"]["board"]["cells"][1][1]["entity"], 
        "piece_p1",
        "Destination cell should have the piece"
    );
}

#[test]
fn test_multi_step_formatting_for_ui_vs_action() {
    use bluefelt_core::engine::verbs::multi_step::{
        apply_multi_step_templates, apply_multi_step_templates_no_format
    };
    
    let mut stored_values = HashMap::new();
    stored_values.insert("from".to_string(), json!("/zones/board/cells/2/1"));
    stored_values.insert("to".to_string(), json!("/zones/board/cells/0/2"));
    
    // UI formatting should convert paths to coordinates
    let ui_prompt = json!("Move from {from} to {to}?");
    let ui_result = apply_multi_step_templates(&ui_prompt, &stored_values);
    assert_eq!(ui_result.as_str().unwrap(), "Move from (1, 2) to (2, 0)?");
    
    // Action formatting should preserve paths
    let action_args = json!({
        "from": "{from}",
        "to": "{to}"
    });
    let action_result = apply_multi_step_templates_no_format(&action_args, &stored_values);
    assert_eq!(action_result["from"], "/zones/board/cells/2/1");
    assert_eq!(action_result["to"], "/zones/board/cells/0/2");
}