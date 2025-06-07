use serde_json::{json, Value};

pub struct TestStateGenerator;

impl TestStateGenerator {
    pub fn generate_state(test_type: &str, params: &Value) -> Result<Value, String> {
        match test_type {
            "board-zone" => Self::generate_board_state(params),
            "card-zone" => Self::generate_card_state(params),
            "choice-zone" => Self::generate_choice_state(params),
            "deck-zone" => Self::generate_deck_state(params),
            "composite" => Self::generate_composite_state(params),
            _ => Err(format!("Unknown test type: {}", test_type))
        }
    }

    fn generate_board_state(params: &Value) -> Result<Value, String> {
        let grid_size = params["gridSize"]
            .as_array()
            .and_then(|a| Some((a[0].as_u64()? as usize, a[1].as_u64()? as usize)))
            .unwrap_or((3, 3));
        
        let fill_pattern = params["fillPattern"].as_str().unwrap_or("empty");
        let entity_types = params["entityTypes"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["mark_p1", "mark_p2"]);
        
        let action_pattern = params["actionPattern"].as_str().unwrap_or("none");
        
        // Create board zone
        let mut board = vec![vec![Value::Null; grid_size.1]; grid_size.0];
        
        // Apply fill pattern
        match fill_pattern {
            "checkerboard" => {
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        if (row + col) % 2 == 0 {
                            board[row][col] = json!({
                                "entity": entity_types[0],
                                "owner": "p1"
                            });
                        } else {
                            board[row][col] = json!({
                                "entity": entity_types[1 % entity_types.len()],
                                "owner": "p2"
                            });
                        }
                    }
                }
            },
            "full" => {
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        let entity_idx = (row * grid_size.1 + col) % entity_types.len();
                        board[row][col] = json!({
                            "entity": entity_types[entity_idx],
                            "owner": if entity_idx == 0 { "p1" } else { "p2" }
                        });
                    }
                }
            },
            "corners" => {
                board[0][0] = json!({"entity": entity_types[0], "owner": "p1"});
                board[0][grid_size.1-1] = json!({"entity": entity_types[1 % entity_types.len()], "owner": "p2"});
                board[grid_size.0-1][0] = json!({"entity": entity_types[1 % entity_types.len()], "owner": "p2"});
                board[grid_size.0-1][grid_size.1-1] = json!({"entity": entity_types[0], "owner": "p1"});
            },
            "random" => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        if rng.gen::<f64>() > 0.5 {
                            let entity_idx = rng.gen_range(0..entity_types.len());
                            board[row][col] = json!({
                                "entity": entity_types[entity_idx],
                                "owner": if entity_idx == 0 { "p1" } else { "p2" }
                            });
                        }
                    }
                }
            },
            _ => {} // empty
        }
        
        // Create action map
        let mut action_map = json!({});
        match action_pattern {
            "all" => {
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        let path = format!("/zones/board/{}/{}", row, col);
                        action_map[path] = json!({
                            "action": "place",
                            "direction": "Click to place"
                        });
                    }
                }
            },
            "alternating" => {
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        if (row + col) % 2 == 0 {
                            let path = format!("/zones/board/{}/{}", row, col);
                            action_map[path] = json!({
                                "action": "place",
                                "direction": "Click to place"
                            });
                        }
                    }
                }
            },
            "random" => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                for row in 0..grid_size.0 {
                    for col in 0..grid_size.1 {
                        if rng.gen::<f64>() < 0.3 {
                            let path = format!("/zones/board/{}/{}", row, col);
                            action_map[path] = json!({
                                "action": "place",
                                "direction": "Click to place"
                            });
                        }
                    }
                }
            },
            _ => {} // none
        }
        
        Ok(json!({
            "game": {
                "currentPlayer": "p1",
                "phase": "play"
            },
            "zones": {
                "board": board
            },
            "ui": {
                "actionMap": action_map
            },
            "meta": {
                "gameStatus": {
                    "state": "active"
                }
            }
        }))
    }
    
    fn generate_card_state(params: &Value) -> Result<Value, String> {
        let card_count = params["cardCount"].as_u64().unwrap_or(10) as usize;
        let layout = params["layout"].as_str().unwrap_or("hand");
        let selectable = params["selectable"].as_bool().unwrap_or(false);
        let face_down = params["faceDown"].as_bool().unwrap_or(false);
        
        // Generate cards
        let mut cards = Vec::new();
        let suits = ["hearts", "diamonds", "clubs", "spades"];
        let ranks = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
        
        for i in 0..card_count {
            let suit = suits[i % 4];
            let rank = ranks[i % 13];
            cards.push(json!({
                "entity": format!("card_{}_{}", rank, suit),
                "rank": rank,
                "suit": suit,
                "faceDown": face_down,
                "owner": "p1"
            }));
        }
        
        // Create action map if selectable
        let mut action_map = json!({});
        if selectable {
            for i in 0..card_count {
                let path = format!("/zones/hand_p1/{}", i);
                action_map[path] = json!({
                    "action": "select",
                    "direction": "Select card"
                });
            }
        }
        
        Ok(json!({
            "game": {
                "currentPlayer": "p1",
                "phase": "play"
            },
            "zones": {
                "hand_p1": cards,
                "deck": []
            },
            "ui": {
                "actionMap": action_map,
                "layout": {
                    "hand_p1": layout
                }
            },
            "meta": {
                "gameStatus": {
                    "state": "active"
                }
            }
        }))
    }
    
    fn generate_choice_state(params: &Value) -> Result<Value, String> {
        let choice_count = params["choiceCount"].as_u64().unwrap_or(10) as usize;
        let choice_type = params["choiceType"].as_str().unwrap_or("rank");
        let layout = params["layout"].as_str().unwrap_or("list");
        let multi_select = params["multiSelect"].as_bool().unwrap_or(false);
        
        // Generate choices based on type
        let choices: Vec<Value> = match choice_type {
            "rank" => vec!["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"]
                .into_iter()
                .take(choice_count.min(13))
                .map(|r| json!({"value": r, "label": r}))
                .collect(),
            "suit" => vec!["hearts", "diamonds", "clubs", "spades"]
                .into_iter()
                .take(choice_count.min(4))
                .map(|s| json!({"value": s, "label": s.to_uppercase()}))
                .collect(),
            "color" => vec!["red", "black"]
                .into_iter()
                .take(choice_count.min(2))
                .map(|c| json!({"value": c, "label": c.to_uppercase()}))
                .collect(),
            _ => (0..choice_count)
                .map(|i| json!({"value": format!("choice_{}", i), "label": format!("Choice {}", i + 1)}))
                .collect()
        };
        
        Ok(json!({
            "game": {
                "currentPlayer": "p1",
                "phase": "choice"
            },
            "zones": {
                "choices": choices
            },
            "ui": {
                "actionMap": {
                    "/zones/choices": {
                        "action": "choose",
                        "direction": if multi_select { "Select multiple" } else { "Select one" },
                        "multiSelect": multi_select
                    }
                },
                "layout": {
                    "choices": layout
                }
            },
            "meta": {
                "gameStatus": {
                    "state": "active"
                }
            }
        }))
    }
    
    fn generate_deck_state(params: &Value) -> Result<Value, String> {
        let deck_size = params["deckSize"].as_u64().unwrap_or(52) as usize;
        let show_count = params["showCount"].as_bool().unwrap_or(true);
        let top_visible = params["topVisible"].as_bool().unwrap_or(false);
        
        // Generate deck
        let mut deck = Vec::new();
        for i in 0..deck_size {
            deck.push(json!({
                "entity": format!("card_{}", i),
                "faceDown": !top_visible || i > 0
            }));
        }
        
        Ok(json!({
            "game": {
                "currentPlayer": "p1",
                "phase": "play"
            },
            "zones": {
                "deck": deck,
                "discard": []
            },
            "ui": {
                "actionMap": {
                    "/zones/deck": {
                        "action": "draw",
                        "direction": "Draw card"
                    }
                },
                "deckDisplay": {
                    "showCount": show_count,
                    "topVisible": top_visible
                }
            },
            "meta": {
                "gameStatus": {
                    "state": "active"
                }
            }
        }))
    }
    
    fn generate_composite_state(params: &Value) -> Result<Value, String> {
        // Combine multiple zone types for complex testing
        let include_board = params["includeBoard"].as_bool().unwrap_or(true);
        let include_cards = params["includeCards"].as_bool().unwrap_or(true);
        let include_resources = params["includeResources"].as_bool().unwrap_or(false);
        
        let mut state = json!({
            "game": {
                "currentPlayer": "p1",
                "phase": "play"
            },
            "zones": {},
            "ui": {
                "actionMap": {}
            },
            "meta": {
                "gameStatus": {
                    "state": "active"
                }
            }
        });
        
        if include_board {
            state["zones"]["board"] = json!(vec![vec![Value::Null; 3]; 3]);
        }
        
        if include_cards {
            state["zones"]["hand_p1"] = json!([
                {"entity": "card_A_hearts", "rank": "A", "suit": "hearts"},
                {"entity": "card_K_spades", "rank": "K", "suit": "spades"}
            ]);
        }
        
        if include_resources {
            state["zones"]["resources"] = json!({
                "wood": 5,
                "stone": 3,
                "gold": 1
            });
        }
        
        Ok(state)
    }
}

// HTTP handler
pub async fn get_test_state(
    axum::extract::Path(test_type): axum::extract::Path<String>,
    axum::Json(params): axum::Json<Value>,
) -> Result<axum::Json<Value>, axum::http::StatusCode> {
    match TestStateGenerator::generate_state(&test_type, &params) {
        Ok(state) => Ok(axum::Json(state)),
        Err(_) => Err(axum::http::StatusCode::BAD_REQUEST)
    }
}