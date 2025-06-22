//! Go Fish configuration tests
//! 
//! These tests verify the Go Fish game is properly configured

use bluefelt_core::bundle::BundleMap;
use serde_json::json;

#[test]
fn test_go_fish_bundle_structure() {
    // Load the Go Fish bundle
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Go Fish not found");
    
    // Verify manifest
    assert_eq!(bundle.manifest.game_id, "go-fish");
    assert_eq!(bundle.manifest.metadata.players.min, 2);
    assert_eq!(bundle.manifest.metadata.players.max, 4);
    
    // Verify entities exist
    let entities = bundle.entities.as_array().expect("entities should be array");
    assert_eq!(entities.len(), 56, "Should have 52 cards + 4 deck entities");
    
    // Verify zones
    let zones = bundle.zones.as_array().expect("zones should be array");
    let zone_ids: Vec<&str> = zones.iter()
        .filter_map(|z| z["id"].as_str())
        .collect();
    
    assert!(zone_ids.contains(&"pool"), "Should have pool zone");
    assert!(zone_ids.contains(&"hand_{player}"), "Should have hand zone template");
    assert!(zone_ids.contains(&"pairs_{player}"), "Should have pairs zone template");
    assert!(zone_ids.contains(&"choice_{player}"), "Should have choice zone template");
    
    // Verify key actions exist
    let actions = bundle.actions.as_array().expect("actions should be array");
    let action_ids: Vec<&str> = actions.iter()
        .filter_map(|a| a["id"].as_str())
        .collect();
    
    // Dealing
    assert!(action_ids.contains(&"dealCards"), "Should have dealCards action");
    
    // Core gameplay
    assert!(action_ids.contains(&"selectRank"), "Should have selectRank action");
    assert!(action_ids.contains(&"selectPlayer"), "Should have selectPlayer action");
    assert!(action_ids.contains(&"transferCards"), "Should have transferCards action");
    assert!(action_ids.contains(&"goFish"), "Should have goFish action");
    
    // Pair formation
    assert!(action_ids.contains(&"checkForBooks"), "Should have checkForBooks action");
    
    // Turn management
    assert!(action_ids.contains(&"advanceTurn"), "Should have advanceTurn action");
    
    // Game end
    assert!(action_ids.contains(&"checkGameEnd"), "Should have checkGameEnd action");
    assert!(action_ids.contains(&"calculateWinner"), "Should have calculateWinner action");
}

#[test]
fn test_go_fish_phase_configuration() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Go Fish not found");
    
    // Verify phases
    let phases = bundle.phases.as_array().expect("phases should be array");
    
    // Go Fish has a nested phase structure with "game" as the top-level phase
    let game_phase = phases.iter()
        .find(|p| p["id"].as_str() == Some("game"))
        .expect("Should have game phase");
    
    let nested_phases = game_phase["phases"].as_array()
        .expect("game phase should have nested phases");
    
    let phase_ids: Vec<&str> = nested_phases.iter()
        .filter_map(|p| p["id"].as_str())
        .collect();
    
    assert!(phase_ids.contains(&"dealing"), "Should have dealing phase");
    assert!(phase_ids.contains(&"selectingRank"), "Should have selectingRank phase");
    assert!(phase_ids.contains(&"selectingPlayer"), "Should have selectingPlayer phase");
    assert!(phase_ids.contains(&"responding"), "Should have responding phase");
    assert!(phase_ids.contains(&"checkingPairs"), "Should have checkingPairs phase");
    assert!(phase_ids.contains(&"fishing"), "Should have fishing phase");
    assert!(phase_ids.contains(&"gameOver"), "Should have gameOver phase");
}

#[test]
fn test_go_fish_action_flow() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Go Fish not found");
    
    let actions = bundle.actions.as_array().expect("actions should be array");
    
    // Test selectRank action
    let select_rank = actions.iter()
        .find(|a| a["id"].as_str() == Some("selectRank"))
        .expect("selectRank action should exist");
    
    assert_eq!(select_rank["uses"].as_str(), Some("setState"));
    assert_eq!(
        select_rank["with"]["path"].as_str(), 
        Some("/selection/selectedRank"),
        "Should set selectedRank"
    );
    
    // Verify it has a then action
    assert!(select_rank["then"].is_array(), "selectRank should have then actions");
    let then_action = &select_rank["then"][0];
    assert_eq!(
        then_action["action"].as_str(),
        Some("transitionToSelectingPlayer"),
        "Should transition to selecting player"
    );
    
    // Test goFish action  
    let go_fish = actions.iter()
        .find(|a| a["id"].as_str() == Some("goFish"))
        .expect("goFish action should exist");
    
    assert_eq!(go_fish["uses"].as_str(), Some("draw"));
    assert_eq!(go_fish["with"]["from"].as_str(), Some("/zones/pool"));
    assert_eq!(go_fish["with"]["to"].as_str(), Some("/zones/hand_{player}"));
}

#[test]
fn test_go_fish_logging_configuration() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles.get_latest("go-fish").expect("Go Fish not found");
    
    let actions = bundle.actions.as_array().expect("actions should be array");
    
    // Check that key actions have log templates
    let actions_with_logs = [
        ("dealCards", "Dealing cards to all players"),
        ("goFish", "{selection.targetPlayer} says 'Go Fish!' - {player} draws a card"),
        ("advanceTurn", ""),
    ];
    
    for (action_id, expected_log) in &actions_with_logs {
        let action = actions.iter()
            .find(|a| a["id"].as_str() == Some(*action_id))
            .expect(&format!("{} action should exist", action_id));
        
        let log_template = action["ui"]["logTemplate"].as_str();
        assert_eq!(
            log_template, 
            Some(*expected_log),
            "{} should have correct log template", 
            action_id
        );
    }
}