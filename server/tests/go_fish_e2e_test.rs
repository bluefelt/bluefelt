use bluefelt_core::{
    bundle::Bundle,
    engine::{state::initialize_state, patches::process_phases},
    lobby::{process_action, generate_action_map},
    message_format::{IncomingMessage, OutgoingMessage},
};
use serde_json::{json, Value};
use std::fs;

#[tokio::test]
async fn test_go_fish_complete_game() {
    // Load Go Fish bundle
    let bundle_path = "../bundles/go-fish/1.0";
    let bundle_json = fs::read_to_string(format!("{}/bundle.json", bundle_path))
        .expect("Failed to read bundle.json");
    let bundle: Bundle = serde_json::from_str(&bundle_json)
        .expect("Failed to parse bundle");

    // Initialize game state
    let players = vec![
        json!({"id": "p1", "name": "Alice"}),
        json!({"id": "p2", "name": "Bob"}),
    ];
    let mut state = initialize_state(&bundle, players);
    
    // Process initial phases (dealing)
    let initial_patches = process_phases(&bundle, &mut state).unwrap();
    println!("Initial patches: {}", initial_patches.len());
    
    // Verify cards were dealt
    let hand_p1 = &state["zones"]["hand_p1"]["items"];
    let hand_p2 = &state["zones"]["hand_p2"]["items"];
    assert_eq!(hand_p1.as_array().unwrap().len(), 7, "Player 1 should have 7 cards");
    assert_eq!(hand_p2.as_array().unwrap().len(), 7, "Player 2 should have 7 cards");
    
    // Verify we're in the selectingRank phase
    assert_eq!(state["phases"]["game"], "selectingRank");
    assert_eq!(state["currentPlayer"], "p1");
    
    // Test 1: Player 1 selects a rank
    let select_rank_msg = IncomingMessage {
        action: "selectRank".to_string(),
        args: Some(json!({
            "rank": "10",
            "player": "p1"
        })),
    };
    
    let result = process_action(&bundle, &mut state, "p1", &select_rank_msg);
    assert!(result.is_ok(), "selectRank should succeed");
    
    // Should be in selectingPlayer phase
    assert_eq!(state["phases"]["game"], "selectingPlayer");
    assert_eq!(state["selection"]["selectedRank"], "10");
    
    // Test 2: Player 1 selects player 2 to ask
    let select_player_msg = IncomingMessage {
        action: "selectPlayer".to_string(),
        args: Some(json!({
            "targetPlayer": "p2",
            "player": "p1"
        })),
    };
    
    let result = process_action(&bundle, &mut state, "p1", &select_player_msg);
    assert!(result.is_ok(), "selectPlayer should succeed");
    
    // Should be in responding phase
    assert_eq!(state["phases"]["game"], "responding");
    
    // Process responding phase
    let responding_patches = process_phases(&bundle, &mut state).unwrap();
    println!("Responding patches: {}", responding_patches.len());
    
    // Check if we're in fishing or back to selectingRank
    let current_phase = state["phases"]["game"].as_str().unwrap();
    println!("Current phase after responding: {}", current_phase);
    
    if current_phase == "fishing" {
        // Player 2 didn't have the requested rank
        println!("Go Fish scenario");
        
        // Process fishing phase
        let fishing_patches = process_phases(&bundle, &mut state).unwrap();
        println!("Fishing patches: {}", fishing_patches.len());
        
        // Should either continue turn or switch turns
        let current_player = state["currentPlayer"].as_str().unwrap();
        println!("Current player after fishing: {}", current_player);
    } else if current_phase == "selectingRank" {
        // Player 2 had the requested rank
        println!("Transfer cards scenario");
        
        // Player 1 continues their turn
        assert_eq!(state["currentPlayer"], "p1");
    }
    
    // Test 3: Verify action map generation
    let action_map = generate_action_map(&bundle, &state, "p1");
    println!("Action map for p1: {:?}", action_map);
    
    // Test 4: Continue playing until game ends
    let mut turn_count = 0;
    let max_turns = 50;
    
    while turn_count < max_turns {
        turn_count += 1;
        
        let current_player = state["currentPlayer"].as_str().unwrap();
        let current_phase = state["phases"]["game"].as_str().unwrap();
        
        println!("Turn {}: Player {} in phase {}", turn_count, current_player, current_phase);
        
        // Check for game over
        if current_phase == "gameOver" {
            println!("Game ended after {} turns", turn_count);
            break;
        }
        
        // Get action map for current player
        let action_map = generate_action_map(&bundle, &state, current_player);
        
        if action_map.is_empty() {
            println!("No actions available for {}", current_player);
            break;
        }
        
        // Simulate player actions based on phase
        match current_phase {
            "selectingRank" => {
                // Select first available rank
                if let Some((_, action)) = action_map.iter().find(|(k, _)| k.contains("/ranks/")) {
                    if let Some(rank) = action["rank"].as_str() {
                        let msg = IncomingMessage {
                            action: "selectRank".to_string(),
                            args: Some(json!({
                                "rank": rank,
                                "player": current_player
                            })),
                        };
                        process_action(&bundle, &mut state, current_player, &msg).unwrap();
                    }
                }
            },
            "selectingPlayer" => {
                // Select the other player
                let target = if current_player == "p1" { "p2" } else { "p1" };
                let msg = IncomingMessage {
                    action: "selectPlayer".to_string(),
                    args: Some(json!({
                        "targetPlayer": target,
                        "player": current_player
                    })),
                };
                process_action(&bundle, &mut state, current_player, &msg).unwrap();
            },
            _ => {
                // Process phase enter actions
                let phase_patches = process_phases(&bundle, &mut state).unwrap();
                if phase_patches.is_empty() {
                    println!("No phase patches, might be stuck");
                    break;
                }
            }
        }
    }
    
    // Verify game ended properly
    if state["phases"]["game"] == "gameOver" {
        let game_status = &state["gameStatus"];
        assert_eq!(game_status["state"], "ended");
        assert!(game_status["winner"].is_string() || game_status["winner"].is_array());
        println!("Game ended with winner: {:?}", game_status["winner"]);
    }
}

#[tokio::test]
async fn test_go_fish_card_transfer() {
    // Load bundle and initialize state
    let bundle_path = "../bundles/go-fish/1.0";
    let bundle_json = fs::read_to_string(format!("{}/bundle.json", bundle_path))
        .expect("Failed to read bundle.json");
    let bundle: Bundle = serde_json::from_str(&bundle_json)
        .expect("Failed to parse bundle");

    let players = vec![
        json!({"id": "p1", "name": "Alice"}),
        json!({"id": "p2", "name": "Bob"}),
    ];
    let mut state = initialize_state(&bundle, players);
    
    // Manually set up a scenario where p2 has 10s
    state["zones"]["hand_p1"]["items"] = json!([
        {"entity": "card_hearts_9"},
        {"entity": "card_spades_9"},
        {"entity": "card_clubs_k"},
        {"entity": "card_diamonds_k"},
    ]);
    
    state["zones"]["hand_p2"]["items"] = json!([
        {"entity": "card_hearts_10"},
        {"entity": "card_spades_10"},
        {"entity": "card_clubs_a"},
        {"entity": "card_diamonds_a"},
    ]);
    
    state["phases"]["game"] = json!("selectingRank");
    state["currentPlayer"] = json!("p1");
    
    // Player 1 asks for 10s
    let select_rank_msg = IncomingMessage {
        action: "selectRank".to_string(),
        args: Some(json!({
            "rank": "10",
            "player": "p1"
        })),
    };
    process_action(&bundle, &mut state, "p1", &select_rank_msg).unwrap();
    
    let select_player_msg = IncomingMessage {
        action: "selectPlayer".to_string(),
        args: Some(json!({
            "targetPlayer": "p2",
            "player": "p1"
        })),
    };
    process_action(&bundle, &mut state, "p1", &select_player_msg).unwrap();
    
    // Process responding phase
    process_phases(&bundle, &mut state).unwrap();
    
    // Verify cards were transferred
    let hand_p1 = &state["zones"]["hand_p1"]["items"].as_array().unwrap();
    let hand_p2 = &state["zones"]["hand_p2"]["items"].as_array().unwrap();
    
    // P1 should have gained 2 cards (the 10s)
    assert_eq!(hand_p1.len(), 6, "P1 should have 6 cards after transfer");
    assert_eq!(hand_p2.len(), 2, "P2 should have 2 cards after transfer");
    
    // P1 should still be the current player
    assert_eq!(state["currentPlayer"], "p1");
    assert_eq!(state["phases"]["game"], "selectingRank");
}

#[tokio::test]
async fn test_go_fish_pair_formation() {
    // Load bundle and initialize state
    let bundle_path = "../bundles/go-fish/1.0";
    let bundle_json = fs::read_to_string(format!("{}/bundle.json", bundle_path))
        .expect("Failed to read bundle.json");
    let bundle: Bundle = serde_json::from_str(&bundle_json)
        .expect("Failed to parse bundle");

    let players = vec![
        json!({"id": "p1", "name": "Alice"}),
        json!({"id": "p2", "name": "Bob"}),
    ];
    let mut state = initialize_state(&bundle, players);
    
    // Set up a scenario where p1 will form a pair
    state["zones"]["hand_p1"]["items"] = json!([
        {"entity": "card_hearts_10"},
        {"entity": "card_spades_9"},
        {"entity": "card_clubs_k"},
    ]);
    
    state["zones"]["hand_p2"]["items"] = json!([
        {"entity": "card_diamonds_10"},
        {"entity": "card_clubs_10"},
        {"entity": "card_clubs_a"},
    ]);
    
    state["phases"]["game"] = json!("selectingRank");
    state["currentPlayer"] = json!("p1");
    
    // Player 1 asks for 10s (will get them and form a pair)
    let select_rank_msg = IncomingMessage {
        action: "selectRank".to_string(),
        args: Some(json!({
            "rank": "10",
            "player": "p1"
        })),
    };
    process_action(&bundle, &mut state, "p1", &select_rank_msg).unwrap();
    
    let select_player_msg = IncomingMessage {
        action: "selectPlayer".to_string(),
        args: Some(json!({
            "targetPlayer": "p2",
            "player": "p1"
        })),
    };
    process_action(&bundle, &mut state, "p1", &select_player_msg).unwrap();
    
    // Process responding phase
    process_phases(&bundle, &mut state).unwrap();
    
    // Verify pair was formed
    let pairs_p1 = &state["zones"]["pairs_p1"]["items"].as_array().unwrap();
    assert!(pairs_p1.len() >= 2, "P1 should have formed a pair");
    
    // P1's hand should have fewer cards
    let hand_p1 = &state["zones"]["hand_p1"]["items"].as_array().unwrap();
    assert!(hand_p1.len() <= 3, "P1 should have fewer cards after forming pair");
}