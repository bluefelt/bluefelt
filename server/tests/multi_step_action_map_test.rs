use serde_json::json;
use bluefelt_server::test_helpers::{create_test_server, TestClient};

#[tokio::test]
async fn test_multi_step_action_map_visibility() {
    let server = create_test_server().await;
    
    // Create a test game with multi-step actions
    let game_bundle = json!({
        "game_id": "test-multistep",
        "manifest": {
            "metadata": {
                "name": "Test Multi-Step Game",
                "players": {"min": 2, "max": 2}
            }
        },
        "actions": [
            {
                "id": "moveEntity",
                "type": "multiStep",
                "isMultiStep": true,
                "cancellable": true,
                "stateStore": ["selectedEntity", "targetLocation"],
                "steps": [
                    {
                        "id": "selectEntity",
                        "as": "bf.selectEntity",
                        "with": {"source": "board"},
                        "store": "selectedEntity"
                    },
                    {
                        "id": "selectTarget",
                        "as": "bf.selectMapSpace", 
                        "with": {"zone": "board"},
                        "store": "targetLocation"
                    }
                ],
                "result": {
                    "as": "bf.moveEntity",
                    "with": {
                        "entity": "{selectedEntity}",
                        "to": "{targetLocation}"
                    }
                }
            }
        ],
        "phases": [{
            "id": "play",
            "type": "standard"
        }],
        "entities": {},
        "zones": [{
            "id": "board",
            "type": "grid",
            "dimensions": {"rows": 3, "cols": 3}
        }]
    });
    
    // Create lobby
    let lobby_id = server.create_lobby_with_bundle("test-multistep", game_bundle).await;
    
    // Connect two players
    let mut player1 = TestClient::connect(&server, &lobby_id, "p1").await;
    let mut player2 = TestClient::connect(&server, &lobby_id, "p2").await;
    
    // Start game
    player1.send_message(json!({"action": "start_game"})).await;
    
    // Wait for game to start
    let msg = player1.receive_message().await;
    assert_eq!(msg["type"], "gameStarted");
    
    // Get initial state
    let welcome = player1.receive_message().await;
    assert_eq!(welcome["type"], "welcome");
    
    // Player 1 initiates multi-step action
    player1.send_message(json!({
        "action": "moveEntity"
    })).await;
    
    // Check that multi-step state is created
    let update = player1.receive_message().await;
    assert_eq!(update["type"], "diff");
    
    let patches = update["patch"].as_array().unwrap();
    
    // Find the multi-step state patch
    let multi_step_patch = patches.iter()
        .find(|p| p["path"] == "/ui/multiStepState")
        .expect("Should have multi-step state patch");
    
    let multi_step_state = &multi_step_patch["value"];
    assert_eq!(multi_step_state["actionId"], "moveEntity");
    
    // Check that step action map exists
    assert!(multi_step_state["stepActionMap"].is_object());
    let step_action_map = multi_step_state["stepActionMap"].as_object().unwrap();
    assert!(!step_action_map.is_empty(), "Step action map should not be empty");
    
    // Check that regular action map is cleared for p1
    let action_map_patch = patches.iter()
        .find(|p| p["path"] == "/ui/actionMap")
        .expect("Should have action map patch");
    
    let action_map = action_map_patch["value"].as_object().unwrap();
    let p1_actions = action_map["p1"].as_object().unwrap();
    assert!(p1_actions.is_empty(), "Regular action map should be empty for p1 during multi-step");
    
    println!("Multi-step state: {}", serde_json::to_string_pretty(multi_step_state).unwrap());
    println!("Action map: {}", serde_json::to_string_pretty(action_map).unwrap());
}