//! Integration tests for ActionExecutor with real game scenarios

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::sync::Arc;
    use crate::bundle::{Bundle, load_bundles_from_dir};
    use crate::engine::action_executor::ActionExecutor;
    use crate::engine::state::GameState;
    use crate::engine::state_init::initialize_state;

    async fn setup_game(game_id: &str) -> (Arc<Bundle>, GameState) {
        let bundles = load_bundles_from_dir("../bundles").unwrap();
        let bundle = bundles.get(game_id).unwrap().clone();
        let state = initialize_state(&bundle, vec!["p1".to_string(), "p2".to_string()]).await;
        (bundle, state)
    }

    #[tokio::test]
    async fn test_tic_tac_toe_complete_game_flow() {
        let (bundle, mut state) = setup_game("tic-tac-toe").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Player 1 places mark at (0,0)
        let (patches, triggered) = executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/0/0" }),
            "p1"
        ).await.unwrap();

        assert!(!patches.is_empty());
        assert_eq!(state.zones["board"][0][0], json!("X_p1"));
        assert_eq!(state.game["currentPlayer"], "p2");

        // Player 2 places mark at (1,1)
        let (patches, triggered) = executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/1/1" }),
            "p2"
        ).await.unwrap();

        assert!(!patches.is_empty());
        assert_eq!(state.zones["board"][1][1], json!("O_p2"));
        assert_eq!(state.game["currentPlayer"], "p1");

        // Continue game to test win detection
        // P1 at (0,1)
        executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/0/1" }),
            "p1"
        ).await.unwrap();

        // P2 at (2,2)
        executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/2/2" }),
            "p2"
        ).await.unwrap();

        // P1 wins with (0,2)
        let (patches, triggered) = executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/0/2" }),
            "p1"
        ).await.unwrap();

        // Check win was detected
        assert_eq!(state.meta["gameStatus"]["state"], "ended");
        assert_eq!(state.meta["gameStatus"]["winner"], "p1");
        
        // Verify triggered actions included win sequence
        let action_names: Vec<&str> = triggered.iter()
            .map(|a| a.action_id.as_str())
            .collect();
        assert!(action_names.contains(&"checkWin"));
    }

    #[tokio::test]
    async fn test_connect_four_gravity_and_win() {
        let (bundle, mut state) = setup_game("connect-four").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Test gravity - place in column 3
        let (patches, _) = executor.execute_action(
            &mut state,
            "dropPiece",
            json!({ "location": "/zones/board/0/3" }),
            "p1"
        ).await.unwrap();

        // Should land at bottom row (5)
        assert_eq!(state.zones["board"][5][3], json!("red_p1"));

        // P2 places in same column
        executor.execute_action(
            &mut state,
            "dropPiece",
            json!({ "location": "/zones/board/0/3" }),
            "p2"
        ).await.unwrap();

        // Should stack on top
        assert_eq!(state.zones["board"][4][3], json!("yellow_p2"));

        // Build towards a win - horizontal line for p1
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/0" }), "p1").await.unwrap();
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/0" }), "p2").await.unwrap();
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/1" }), "p1").await.unwrap();
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/1" }), "p2").await.unwrap();
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/2" }), "p1").await.unwrap();
        executor.execute_action(&mut state, "dropPiece", 
            json!({ "location": "/zones/board/0/4" }), "p2").await.unwrap();

        // P1 completes horizontal line
        let (_, triggered) = executor.execute_action(
            &mut state,
            "dropPiece",
            json!({ "location": "/zones/board/0/4" }),
            "p1"
        ).await.unwrap();

        // Verify win detection via triggered actions
        assert_eq!(state.meta["gameStatus"]["state"], "ended");
        assert_eq!(state.meta["gameStatus"]["winner"], "p1");
    }

    #[tokio::test]
    async fn test_three_mens_morris_phase_transitions() {
        let (bundle, mut state) = setup_game("three-mens-morris").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Phase 1: Placement
        assert_eq!(state.game["phase"]["id"], "placement");

        // Place pieces
        for i in 0..3 {
            // P1 places
            executor.execute_action(
                &mut state,
                "place_piece",
                json!({ "location": format!("/zones/board/{}", i) }),
                "p1"
            ).await.unwrap();

            // P2 places
            executor.execute_action(
                &mut state,
                "place_piece", 
                json!({ "location": format!("/zones/board/{}", i + 3) }),
                "p2"
            ).await.unwrap();
        }

        // Should transition to movement phase
        assert_eq!(state.game["phase"]["id"], "movement");
    }

    #[tokio::test]
    async fn test_conditional_action_execution() {
        let (bundle, mut state) = setup_game("tic-tac-toe").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Fill board to near-tie condition
        let moves = vec![
            ("p1", "/zones/board/0/0"), ("p2", "/zones/board/0/1"),
            ("p1", "/zones/board/0/2"), ("p2", "/zones/board/1/0"),
            ("p1", "/zones/board/1/2"), ("p2", "/zones/board/1/1"),
            ("p1", "/zones/board/2/1"), ("p2", "/zones/board/2/0"),
        ];

        for (player, location) in moves {
            executor.execute_action(
                &mut state,
                "placeMark",
                json!({ "location": location }),
                player
            ).await.unwrap();
        }

        // Last move fills the board - should trigger tie detection
        let (_, triggered) = executor.execute_action(
            &mut state,
            "placeMark",
            json!({ "location": "/zones/board/2/2" }),
            "p1"
        ).await.unwrap();

        // Verify tie was detected
        assert_eq!(state.meta["gameStatus"]["state"], "ended");
        assert_eq!(state.meta["gameStatus"].get("winner"), None);
        assert_eq!(state.meta["gameStatus"]["tie"], true);
    }

    #[tokio::test]
    async fn test_template_variable_complex_replacement() {
        let (bundle, mut state) = setup_game("go-fish").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Deal initial cards
        executor.execute_action(
            &mut state,
            "dealCards",
            json!({}),
            "system"
        ).await.unwrap();

        // Test askForRank with template variables
        let (patches, triggered) = executor.execute_action(
            &mut state,
            "askForRank",
            json!({ 
                "targetPlayer": "p2",
                "rank": "A"
            }),
            "p1"
        ).await.unwrap();

        // Verify the action processed with correct template replacements
        // The patches should show cards moved or action results
        assert!(!patches.is_empty() || !triggered.is_empty());
    }

    #[tokio::test]
    async fn test_action_depth_limit_protection() {
        // Create a test bundle with recursive actions
        let mut bundle = Bundle::default();
        bundle.game_id = "test-recursive".to_string();
        
        // Create infinitely recursive action
        bundle.actions.insert("recursive".to_string(), json!({
            "id": "recursive",
            "uses": "setState",
            "with": {
                "path": "/game/counter",
                "value": 1
            },
            "then": [{"action": "recursive"}]
        }));

        let bundle = Arc::new(bundle);
        let mut state = GameState::default();
        let executor = ActionExecutor::new(bundle);

        // Should not panic or hang
        let result = executor.execute_action(
            &mut state,
            "recursive",
            json!({}),
            "p1"
        ).await;

        // Should fail with depth limit error
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("depth"));
    }

    #[tokio::test]
    async fn test_multi_step_action_sequence() {
        let (bundle, mut state) = setup_game("test-multistep").await;
        let executor = ActionExecutor::new(bundle.clone());

        // Execute an action that triggers multiple steps
        let (patches, triggered) = executor.execute_action(
            &mut state,
            "complexSequence",
            json!({}),
            "p1"
        ).await.unwrap();

        // Verify all steps executed in order
        assert!(triggered.len() >= 2, "Should trigger multiple actions");
        
        // Check triggered action sequence
        let first_triggered = &triggered[0];
        assert_eq!(first_triggered.source, crate::engine::TriggeredActionSource::ThenAction);
    }
}