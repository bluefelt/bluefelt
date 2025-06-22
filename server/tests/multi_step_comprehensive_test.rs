use serde_json::{json, Value};
use std::collections::HashMap;
use bluefelt_core::engine::verbs::multi_step::{
    parse_multi_step_action, execute_multi_step_next, execute_multi_step_selection, 
    execute_multi_step_finalize, MultiStepState, MultiStepResponse
};
use bluefelt_core::bundle::{Bundle, Manifest, ManifestMetadata, PlayersRange};

/// Test multi-step action parsing from YAML definition
#[test]
fn test_parse_multi_step_action() {
    let action_json = json!({
        "id": "movePiece",
        "type": "multiStep",
        "cancellable": true,
        "confirmBeforeFinalizing": true,
        "ui": {
            "confirmationPrompt": "Move piece from {selectedPiece} to {destination}?"
        },
        "stateStore": ["selectedPiece", "destination"],
        "steps": [
            {
                "id": "selectPiece",
                "as": "bf.selectEntity",
                "with": {"source": "board"},
                "store": "selectedPiece"
            },
            {
                "id": "selectDestination", 
                "as": "bf.selectMapSpace",
                "with": {"zone": "board"},
                "store": "destination"
            }
        ],
        "result": {
            "as": "bf.moveEntity",
            "with": {
                "from": "{selectedPiece}",
                "to": "{destination}"
            }
        }
    });

    let action = parse_multi_step_action(&action_json).unwrap();
    assert_eq!(action.id, "movePiece");
    assert_eq!(action.steps.len(), 2);
    assert_eq!(action.state_store.len(), 2);
    assert!(action.cancellable);
    assert!(action.confirm_before_finalizing);
    assert_eq!(action.steps[0].id, "selectPiece");
    assert_eq!(action.steps[0].action_type, "bf.selectEntity");
    assert_eq!(action.steps[1].id, "selectDestination");
    assert_eq!(action.steps[1].action_type, "bf.selectMapSpace");
}

/// Test multi-step state progression through steps
#[test]
fn test_multi_step_state_progression() {
    let bundle = create_test_bundle();
    let mut state = create_test_game_state();
    
    let action_def = parse_multi_step_action(&create_test_multi_step_action()).unwrap();
    let mut multi_step_state = MultiStepState {
        action_id: "movePiece".to_string(),
        current_step: 0,
        stored_values: HashMap::new(),
        can_cancel: true,
        deferred_logs: Vec::new(),
        created_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
    };

    // Test first step (select piece)
    let response = execute_multi_step_next(&bundle, &mut state, "p1", &mut multi_step_state, &action_def).unwrap();
    match response {
        MultiStepResponse::StepReady { step_id, action_type, .. } => {
            assert_eq!(step_id, "selectPiece");
            assert_eq!(action_type, "bf.selectEntity");
        }
        _ => panic!("Expected StepReady response"),
    }

    // Simulate piece selection
    let piece_selection = json!({
        "location": "/zones/board/cells/0/0",
        "entity": "token_p1",
        "row": 0,
        "col": 0
    });
    
    let response = execute_multi_step_selection(&bundle, &mut state, "p1", &mut multi_step_state, &action_def, &piece_selection).unwrap();
    match response {
        MultiStepResponse::StepReady { step_id, action_type, .. } => {
            assert_eq!(step_id, "selectDestination");
            assert_eq!(action_type, "bf.selectMapSpace");
        }
        _ => panic!("Expected StepReady response for second step"),
    }

    // Check that piece selection was stored
    assert!(multi_step_state.stored_values.contains_key("selectedPiece"));
    
    // Simulate destination selection
    let destination_selection = json!({
        "location": "/zones/board/cells/2/2",
        "row": 2,
        "col": 2
    });
    
    let response = execute_multi_step_selection(&bundle, &mut state, "p1", &mut multi_step_state, &action_def, &destination_selection).unwrap();
    match response {
        MultiStepResponse::ConfirmationRequired { prompt } => {
            assert!(prompt.contains("Move piece from"));
            assert!(prompt.contains("to"));
        }
        _ => panic!("Expected ConfirmationRequired response"),
    }

    // Check that both selections were stored
    assert!(multi_step_state.stored_values.contains_key("selectedPiece"));
    assert!(multi_step_state.stored_values.contains_key("destination"));
}

/// Test multi-step action finalization
#[test]
fn test_multi_step_finalization() {
    let bundle = create_test_bundle();
    let mut state = create_test_game_state();
    
    let action_def = parse_multi_step_action(&create_test_multi_step_action()).unwrap();
    let multi_step_state = MultiStepState {
        action_id: "movePiece".to_string(),
        current_step: 2,
        stored_values: {
            let mut map = HashMap::new();
            map.insert("selectedPiece".to_string(), json!("/zones/board/cells/0/0"));
            map.insert("destination".to_string(), json!("/zones/board/cells/2/2"));
            map
        },
        can_cancel: true,
        deferred_logs: Vec::new(),
        created_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
    };

    let response = execute_multi_step_finalize(&bundle, &mut state, "p1", &multi_step_state, &action_def).unwrap();
    match response {
        MultiStepResponse::Completed { patches, .. } => {
            assert!(!patches.is_empty(), "Should produce patches for move action");
        }
        _ => panic!("Expected Completed response"),
    }
}

/// Test error conditions
#[test]
fn test_multi_step_error_conditions() {
    let bundle = create_test_bundle();
    let mut state = create_test_game_state();
    
    let action_def = parse_multi_step_action(&create_test_multi_step_action()).unwrap();
    let mut multi_step_state = MultiStepState {
        action_id: "movePiece".to_string(),
        current_step: 10, // Invalid step index
        stored_values: HashMap::new(),
        can_cancel: true,
        deferred_logs: Vec::new(),
        created_at: std::time::SystemTime::now(),
        last_activity: std::time::SystemTime::now(),
    };

    // Test selection on invalid step
    let selection = json!({"location": "0/0"});
    let result = execute_multi_step_selection(&bundle, &mut state, "p1", &mut multi_step_state, &action_def, &selection);
    assert!(result.is_err(), "Should fail when no active step");
}

/// Test template substitution in confirmation prompts
#[test]
fn test_template_substitution() {
    use bluefelt_core::engine::verbs::multi_step::apply_multi_step_templates;
    
    let mut stored_values = HashMap::new();
    stored_values.insert("selectedPiece".to_string(), json!("/zones/board/cells/0/0"));
    stored_values.insert("destination".to_string(), json!("/zones/board/cells/2/2"));

    let template = json!("Move piece from {selectedPiece} to {destination}?");
    let result = apply_multi_step_templates(&template, &stored_values);
    
    assert_eq!(result.as_str().unwrap(), "Move piece from /zones/board/cells/0/0 to /zones/board/cells/2/2?");
}

/// Test template substitution with complex objects
#[test]
fn test_template_substitution_complex_objects() {
    use bluefelt_core::engine::verbs::multi_step::apply_multi_step_templates;
    
    let mut stored_values = HashMap::new();
    stored_values.insert("selectedPiece".to_string(), json!({
        "location": "0/0",
        "row": 0,
        "col": 0,
        "entity": "token_p1"
    }));
    stored_values.insert("destination".to_string(), json!({
        "location": "2/2",
        "row": 2,
        "col": 2
    }));

    let template = json!("Move piece from {selectedPiece} to {destination}?");
    let result = apply_multi_step_templates(&template, &stored_values);
    
    // Should extract location field from objects
    assert_eq!(result.as_str().unwrap(), "Move piece from 0/0 to 2/2?");
}

// Helper functions to create test data

fn create_test_bundle() -> Bundle {
    Bundle {
        game_id: "test-multistep".to_string(),
        manifest: Manifest {
            game_id: "test-multistep".to_string(),
            version: "1.0".to_string(),
            spec_version: "1".to_string(),
            metadata: ManifestMetadata {
                name: "Test Multi-Step".to_string(),
                author: "Test".to_string(),
                description: "Test game for multi-step actions".to_string(),
                players: PlayersRange { min: 2, max: 2 },
            },
            phases: None,
            setup: None,
            zone_groups: None,
        },
        actions: json!([
            {
                "id": "movePiece",
                "type": "multiStep",
                "cancellable": true,
                "confirmBeforeFinalizing": true,
                "ui": {
                    "confirmationPrompt": "Move piece from {selectedPiece} to {destination}?"
                },
                "stateStore": ["selectedPiece", "destination"],
                "steps": [
                    {
                        "id": "selectPiece",
                        "as": "bf.selectEntity",
                        "with": {"source": "board"},
                        "store": "selectedPiece"
                    },
                    {
                        "id": "selectDestination", 
                        "as": "bf.selectMapSpace",
                        "with": {"zone": "board"},
                        "store": "destination"
                    }
                ],
                "result": {
                    "as": "bf.moveEntity",
                    "with": {
                        "from": "{selectedPiece}",
                        "to": "{destination}"
                    }
                }
            }
        ]),
        zones: json!({
            "board": {
                "type": "grid",
                "size": {"rows": 3, "cols": 3}
            }
        }),
        entities: json!([]),
        phases: json!([])
    }
}

fn create_test_game_state() -> Value {
    json!({
        "zones": {
            "board": {
                "type": "grid",
                "cells": [
                    [{"entity": "token_p1"}, null, null],
                    [null, null, null],
                    [null, null, null]
                ]
            }
        },
        "players": [
            {"id": "p1", "name": "Alice"},
            {"id": "p2", "name": "Bob"}
        ],
        "currentPlayer": "p1"
    })
}

fn create_test_multi_step_action() -> Value {
    json!({
        "id": "movePiece",
        "type": "multiStep",
        "cancellable": true,
        "confirmBeforeFinalizing": true,
        "ui": {
            "confirmationPrompt": "Move piece from {selectedPiece} to {destination}?"
        },
        "stateStore": ["selectedPiece", "destination"],
        "steps": [
            {
                "id": "selectPiece",
                "as": "bf.selectEntity",
                "with": {"source": "board"},
                "store": "selectedPiece"
            },
            {
                "id": "selectDestination", 
                "as": "bf.selectMapSpace",
                "with": {"zone": "board"},
                "store": "destination"
            }
        ],
        "result": {
            "as": "bf.moveEntity",
            "with": {
                "from": "{selectedPiece}",
                "to": "{destination}"
            }
        }
    })
}