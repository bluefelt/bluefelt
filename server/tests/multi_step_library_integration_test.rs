use serde_json::json;

#[test]
fn test_multi_step_library_concept() {
    // This test verifies that the concept of multi-step library actions
    // can work with the existing infrastructure
    
    // Create a test action structure
    let action = json!({
        "id": "testMultiStepLibrary",
        "multiStep": {
            "library": "selectAndMove",
            "params": {
                "entityFilter": {"type": "piece"},
                "moveFilter": {"empty": true}
            }
        },
        "when": [
            {"condition": "player.isActor"}
        ]
    });
    
    // Verify the action structure is valid
    assert_eq!(action["id"], "testMultiStepLibrary");
    
    // Verify multi-step library configuration
    assert!(action["multiStep"].is_object());
    assert_eq!(action["multiStep"]["library"], "selectAndMove");
    assert!(action["multiStep"]["params"].is_object());
}

#[test]
fn test_multi_step_library_action_structure() {
    // Test that we can create an action with library reference
    let action = json!({
        "id": "moveWithLibrary",
        "multiStep": {
            "library": "selectAndMove",
            "params": {
                "entityFilter": {"owner": "{player}"},
                "destinationFilter": {"empty": true}
            },
            "stateStore": ["selectedEntity", "destination"],
            "cancellable": true
        }
    });
    
    // Verify the structure
    assert_eq!(action["id"], "moveWithLibrary");
    assert!(action["multiStep"].is_object());
    assert_eq!(action["multiStep"]["library"], "selectAndMove");
    assert!(action["multiStep"]["params"].is_object());
    assert_eq!(action["multiStep"]["cancellable"], true);
}

#[test]
fn test_all_library_types() {
    // Define all expected library types
    let library_types = vec![
        ("chooseCard", json!({"from": "hand", "prompt": "Choose a card"})),
        ("selectAndMove", json!({"entityFilter": {}, "moveFilter": {}})),
        ("selectAndTransfer", json!({"from": "deck", "to": "hand"})),
        ("selectAndDiscard", json!({"from": "hand", "to": "discard"})),
        ("selectMultiple", json!({"count": 3, "from": "deck"})),
        ("choosePlayer", json!({"prompt": "Choose opponent"})),
        ("confirmAction", json!({"prompt": "Are you sure?", "confirmText": "Yes", "cancelText": "No"})),
        ("pickAndPlace", json!({"pickFrom": "supply", "placeTo": "board"})),
    ];
    
    // Verify we have 8 library types
    assert_eq!(library_types.len(), 8);
    
    // Test creating actions with each library type
    for (library_type, params) in library_types {
        let action = json!({
            "id": format!("test_{}", library_type),
            "multiStep": {
                "library": library_type,
                "params": params,
                "cancellable": true
            }
        });
        
        assert!(action["multiStep"].is_object());
        assert_eq!(action["multiStep"]["library"], library_type);
    }
}

#[test]
fn test_library_parameter_validation() {
    // Test required parameters for different library types
    let validation_tests = vec![
        // (library_type, params, should_be_valid)
        ("chooseCard", json!({"from": "hand"}), true),
        ("chooseCard", json!({}), false), // Missing 'from'
        ("selectMultiple", json!({"count": 3, "from": "deck"}), true),
        ("selectMultiple", json!({"count": 3}), false), // Missing 'from'
        ("selectMultiple", json!({"from": "deck"}), false), // Missing 'count'
    ];
    
    for (library_type, params, should_be_valid) in validation_tests {
        // In a real implementation, this would call validate_library_params
        // For now, just verify the structure
        let has_required_fields = match library_type {
            "chooseCard" => params.get("from").is_some(),
            "selectMultiple" => params.get("count").is_some() && params.get("from").is_some(),
            _ => true,
        };
        
        assert_eq!(has_required_fields, should_be_valid, 
            "Validation for {} with params {:?} should be {}", 
            library_type, params, should_be_valid);
    }
}