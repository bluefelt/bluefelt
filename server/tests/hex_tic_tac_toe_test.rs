//! Integration tests for hex tic-tac-toe game

#[cfg(test)]
mod tests {
    use serde_json::json;
    use crate::test_helpers::{TestGame, create_test_server};

    #[tokio::test]
    async fn test_hex_tic_tac_toe_basic_placement() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        // Alice places at center (0,0)
        game.send_action("Alice", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/0,0"
            }
        })).await;

        let state = game.get_state();
        let hex_board = state["zones"]["hex_board"].as_object().unwrap();
        assert_eq!(hex_board.get("0,0").unwrap(), "mark_p1");
        assert_eq!(state["game"]["currentPlayer"], "p2");

        // Bob places at (1,0)
        game.send_action("Bob", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/1,0"
            }
        })).await;

        let state = game.get_state();
        let hex_board = state["zones"]["hex_board"].as_object().unwrap();
        assert_eq!(hex_board.get("1,0").unwrap(), "mark_p2");
        assert_eq!(state["game"]["currentPlayer"], "p1");
    }

    #[tokio::test]
    async fn test_hex_coordinates() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        // Test various hex coordinates are valid
        let valid_coords = vec![
            "0,0",   // center
            "1,0", "-1,0",  // q-axis
            "0,1", "0,-1",  // r-axis
            "1,-1", "-1,1", // s-axis
            "2,0", "-2,0", "0,2", "0,-2", "2,-2", "-2,2", // edge cells
            "1,1", "-1,-1", "2,-1", "-2,1", "1,-2", "-1,2" // remaining cells
        ];

        // Place marks on first few coordinates
        for (i, coord) in valid_coords.iter().enumerate().take(4) {
            let player = if i % 2 == 0 { "Alice" } else { "Bob" };
            game.send_action(player, json!({
                "actionId": "place_mark",
                "args": {
                    "location": format!("/zones/hex_board/{}", coord)
                }
            })).await;

            let state = game.get_state();
            let hex_board = state["zones"]["hex_board"].as_object().unwrap();
            let expected_mark = if i % 2 == 0 { "mark_p1" } else { "mark_p2" };
            assert_eq!(hex_board.get(*coord).unwrap(), expected_mark);
        }
    }

    #[tokio::test]
    async fn test_hex_board_shape() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        let state = game.get_state();
        let hex_board_meta = &state["zones"]["hex_board"];
        
        // Check zone configuration
        assert_eq!(hex_board_meta["type"], "hexgrid");
        assert_eq!(hex_board_meta["shape"], "hexgrid");
        assert_eq!(hex_board_meta["shapeMeta"]["layout"], "flat");
        assert_eq!(hex_board_meta["shapeMeta"]["radius"], 2);
    }

    #[tokio::test]
    async fn test_invalid_hex_placement() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        // Alice places at (0,0)
        game.send_action("Alice", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/0,0"
            }
        })).await;

        // Bob tries to place at same location
        let result = game.try_send_action("Bob", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/0,0"
            }
        })).await;

        assert!(result.is_err() || result.unwrap()["error"].is_string());
        
        // Should still be Bob's turn
        let state = game.get_state();
        assert_eq!(state["game"]["currentPlayer"], "p2");
    }

    #[tokio::test]
    async fn test_hex_board_full_tie() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        // Fill most of the board without winning
        let moves = vec![
            ("Alice", "0,0"), ("Bob", "1,0"),
            ("Alice", "0,1"), ("Bob", "1,-1"),
            ("Alice", "-1,0"), ("Bob", "-1,1"),
            ("Alice", "0,-1"), ("Bob", "2,0"),
            ("Alice", "-2,0"), ("Bob", "0,2"),
            ("Alice", "1,1"), ("Bob", "-1,-1"),
            ("Alice", "2,-2"), ("Bob", "-2,2"),
            ("Alice", "2,-1"), ("Bob", "-2,1"),
            ("Alice", "1,-2"), ("Bob", "-1,2"),
            ("Alice", "0,-2") // 19th move fills the board
        ];

        for (player, coord) in moves {
            game.send_action(player, json!({
                "actionId": "place_mark",
                "args": {
                    "location": format!("/zones/hex_board/{}", coord)
                }
            })).await;
        }

        // Check game ended in tie
        let state = game.get_state();
        assert_eq!(state["meta"]["gameStatus"]["state"], "ended");
        assert_eq!(state["meta"]["gameStatus"]["result"], "tie");
        assert_eq!(state["game"]["phase"], "game_over");
    }

    #[tokio::test]
    async fn test_action_map_shows_valid_hexes() {
        let mut game = TestGame::new("hex-tic-tac-toe", vec!["Alice", "Bob"]).await;
        game.start_game().await;

        // Place a few marks
        game.send_action("Alice", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/0,0"
            }
        })).await;

        game.send_action("Bob", json!({
            "actionId": "place_mark",
            "args": {
                "location": "/zones/hex_board/1,0"
            }
        })).await;

        let state = game.get_state();
        let action_map = &state["ui"]["actionMap"]["p1"];

        // Check that occupied cells are not in action map
        assert!(action_map.get("/zones/hex_board/0,0").is_none());
        assert!(action_map.get("/zones/hex_board/1,0").is_none());

        // Check that empty cells are in action map
        assert!(action_map.get("/zones/hex_board/0,1").is_some());
        assert!(action_map.get("/zones/hex_board/-1,0").is_some());
    }
}