use bluefelt_core::{bundle::BundleMap, engine::{apply_action, load_initial_state_with_rng}};
use serde_json::{json, Value};
use rand::{SeedableRng, rngs::StdRng};

/// Test that client-generated messages work with server action processing
/// This ensures client and server remain compatible
#[test]
fn test_client_server_action_compatibility() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    
    // Test data structure that mimics what the client sends
    struct ClientAction {
        game: &'static str,
        action: &'static str,
        args: Value,
        should_succeed: bool,
        description: &'static str,
    }
    
    let client_actions = vec![
        // Tic-Tac-Toe
        ClientAction {
            game: "tic-tac-toe",
            action: "placeMarker",
            args: json!({
                "location": "/zones/board/cells/0/0",
                "entity": "mark_p1"
            }),
            should_succeed: true,
            description: "Tic-tac-toe place marker",
        },
        
        // Connect Four
        ClientAction {
            game: "connect-four",
            action: "dropDisc",
            args: json!({
                "targetColumn": 1
            }),
            should_succeed: true,
            description: "Connect Four drop disc",
        },
        
        // Three Men's Morris
        ClientAction {
            game: "three-mens-morris",
            action: "placeToken",
            args: json!({
                "target": "/zones/board/cells/0/0",
                "entity": "piece_p1"
            }),
            should_succeed: true,
            description: "Three Men's Morris place token",
        },
        
        // Go Fish
        ClientAction {
            game: "go-fish",
            action: "selectRank",
            args: json!({
                "rank": "A",
                "player": "p1"
            }),
            should_succeed: true,
            description: "Go Fish select rank",
        },
    ];
    
    // Test each client action
    for client_action in client_actions {
        println!("Testing: {}", client_action.description);
        
        let bundle = bundles.get_latest(client_action.game)
            .expect(&format!("Failed to get bundle for {}", client_action.game));
        
        // Create a deterministic RNG for testing
        let mut rng = StdRng::seed_from_u64(12345);
        let mut state = load_initial_state_with_rng(&bundle, None, &mut rng);
        
        // For Go Fish, we need to be in the right phase
        if client_action.game == "go-fish" {
            // Simulate game start - deal cards
            let _ = apply_action(&bundle, &mut state, "p1", &json!({"verb": "dealCards", "args": {}}));
            let _ = apply_action(&bundle, &mut state, "p1", &json!({"verb": "dealToP2", "args": {}}));
            let _ = apply_action(&bundle, &mut state, "p1", &json!({"verb": "beginTurns", "args": {}}));
            let _ = apply_action(&bundle, &mut state, "p1", &json!({"verb": "startTurn", "args": {}}));
        }
        
        // Look up the action definition from the bundle
        let action_def = bundle.actions.as_array()
            .and_then(|actions| actions.iter().find(|a| 
                a.get("id").and_then(|id| id.as_str()) == Some(client_action.action)
            ))
            .expect(&format!("Action '{}' not found in bundle", client_action.action));
        
        // Get the verb from the action definition  
        let verb = action_def.get("uses")
            .and_then(|v| v.as_str())
            .expect(&format!("Action '{}' has no 'uses' field", client_action.action));
        
        // Process 'with' parameters and merge with client args (like the real server does)
        let mut merged_args = client_action.args.clone();
        if let Some(with_params) = action_def.get("with").and_then(|w| w.as_object()) {
            for (key, value) in with_params {
                let processed_value = if let Some(str_val) = value.as_str() {
                    // Replace {args.X} with actual arg values
                    if str_val.starts_with("{args.") && str_val.ends_with("}") {
                        let arg_name = &str_val[6..str_val.len()-1];
                        if let Some(arg_val) = client_action.args.get(arg_name) {
                            arg_val.clone()
                        } else {
                            value.clone()
                        }
                    } else if str_val.contains("{player}") {
                        // Replace {player} with actual player id
                        json!(str_val.replace("{player}", "p1"))
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                };
                
                if let Some(merged_obj) = merged_args.as_object_mut() {
                    merged_obj.insert(key.clone(), processed_value);
                }
            }
        }
        
        
        // Process the action as the server would
        let result = apply_action(
            &bundle,
            &mut state,
            "p1",
            &json!({
                "verb": verb,
                "args": merged_args
            })
        );
        
        if client_action.should_succeed {
            assert!(result.is_ok(), 
                "Action '{}' failed for {}: {:?}", 
                client_action.action, 
                client_action.game,
                result.err()
            );
            
            // Verify patches were generated
            let patches = result.unwrap();
            assert!(!patches.is_empty(), 
                "Action '{}' for {} produced no patches", 
                client_action.action,
                client_action.game
            );
        } else {
            assert!(result.is_err(), 
                "Action '{}' should have failed for {}", 
                client_action.action,
                client_action.game
            );
        }
    }
}

/// Test that action argument names match between client and server
#[test]
fn test_action_argument_consistency() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    
    // Map of expected argument patterns for common actions
    let expected_args = vec![
        ("place", vec!["location", "entity"]),
        ("placeWithGravity", vec!["zone", "column", "entity"]),
        ("selectEntity", vec!["location"]),
        ("moveSelected", vec!["target"]),
        ("setState", vec!["path", "value"]),
        ("draw", vec!["from", "to", "count"]),
    ];
    
    for game_id in bundles.list_games() {
        println!("Checking game: {}", game_id);
        let bundle = bundles.get_latest(&game_id).expect("Failed to get bundle");
        
        if let Some(actions) = bundle.actions.as_array() {
            for action in actions {
                if let Some(uses) = action.get("uses").and_then(|v| v.as_str()) {
                    if let Some(with) = action.get("with").and_then(|v| v.as_object()) {
                        // Check if this action uses template variables
                        for (key, value) in with {
                            if let Some(val_str) = value.as_str() {
                                if val_str.contains("{args.") {
                                    // Extract the argument name
                                    let arg_name = val_str
                                        .trim_start_matches("{args.")
                                        .trim_end_matches("}");
                                    
                                    // For known verbs, verify the argument name is expected
                                    for (verb, expected_arg_names) in &expected_args {
                                        if uses == *verb && key == "location" {
                                            assert!(
                                                expected_arg_names.contains(&"location"),
                                                "Game {} action uses '{}' verb but expects arg '{}' for key '{}'",
                                                game_id, uses, arg_name, key
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}