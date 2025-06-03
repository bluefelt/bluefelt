use bluefelt_core::{bundle::BundleMap, engine::{load_initial_state, process_phases}};
use serde_json::json;

#[test]
fn test_phase_transition_parsing() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles
        .get_latest("three-mens-morris")
        .expect("Failed to get Three Men's Morris bundle");
    let mut state = load_initial_state(&bundle);
    
    // Initial state should have "setup" phase
    println!("Initial phase state: {:?}", state["phases"]);
    assert_eq!(state["phases"]["game"].as_str().unwrap_or(""), "setup");
    
    // Process phases to trigger enterActions
    let patches = process_phases(&bundle, &mut state).expect("Failed to process phases");
    println!("Phase patches: {:?}", patches);
    
    // Check the state after processing phases
    println!("Final phase state: {:?}", state["phases"]);
    
    // The critical test: should be "placement", not "game.placement"
    let game_phase = state["phases"]["game"].as_str().unwrap_or("");
    assert_eq!(game_phase, "placement", 
        "Phase should be 'placement' after parsing 'game.placement' transition");
    
    // Additional verification: should not contain the full dotted notation
    assert_ne!(game_phase, "game.placement", 
        "Phase should not store the full dotted notation 'game.placement'");
    
    // Check that a patch was generated
    assert!(!patches.is_empty(), "Should generate at least one patch for phase transition");
    
    // Verify the patch has the correct value
    let phase_patch = patches.iter()
        .find(|p| p["path"].as_str() == Some("/phases/game"))
        .expect("Should have a phase patch for /phases/game");
    
    assert_eq!(phase_patch["value"].as_str().unwrap(), "placement",
        "Patch should set phase to 'placement', not 'game.placement'");
}

#[test]
fn test_tic_tac_toe_phase_transition() {
    let bundles = BundleMap::load_dir("../bundles").expect("Failed to load bundles");
    let bundle = bundles
        .get_latest("tic-tac-toe")
        .expect("Failed to get Tic Tac Toe bundle");
    let mut state = load_initial_state(&bundle);
    
    // Process phases to trigger enterActions
    let patches = process_phases(&bundle, &mut state).expect("Failed to process phases");
    
    // Tic-tac-toe has: transitionToPhase: game.play
    let game_phase = state["phases"]["game"].as_str().unwrap_or("");
    
    // Should be "play", not "game.play"
    assert_eq!(game_phase, "play", 
        "Tic-tac-toe phase should be 'play' after parsing 'game.play' transition");
    
    assert_ne!(game_phase, "game.play", 
        "Tic-tac-toe phase should not store the full dotted notation 'game.play'");
    
    // Verify the patch has the correct value
    let phase_patch = patches.iter()
        .find(|p| p["path"].as_str() == Some("/phases/game"))
        .expect("Should have a phase patch for /phases/game");
    
    assert_eq!(phase_patch["value"].as_str().unwrap(), "play",
        "Patch should set phase to 'play', not 'game.play'");
}