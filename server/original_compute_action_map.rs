    /// Compute action map for each player based on current state
    /// Returns a map from location (e.g., "/zones/board/0/1") to available actions
    fn compute_action_map(state: &serde_json::Value, bundle: &Bundle) -> serde_json::Map<String, serde_json::Value> {
        let mut player_action_maps = serde_json::Map::new();

        println!("[DEBUG action_map] Checking game state - meta exists: {}, meta.gameStatus exists: {}", 
            state.get("meta").is_some(),
            state.get("meta").and_then(|m| m.get("gameStatus")).is_some()
        );

        // Check if game has ended - check both possible locations
        let game_ended = 
            // Check in meta first
            state.get("meta")
                .and_then(|m| m.get("gameStatus"))
                .and_then(|gs| gs.get("state"))
                .and_then(|s| s.as_str())
                .map(|s| s == "ended")
                .unwrap_or(false) ||
            // Also check at top level (in case patches were applied there)
            state.get("gameStatus")
                .and_then(|gs| gs.get("state"))
                .and_then(|s| s.as_str())
                .map(|s| s == "ended")
                .unwrap_or(false);
                
        if game_ended {
            println!("[DEBUG] Game has ended, returning empty action maps for all players");
            // Game has ended, no moves possible
            if let Some(players) = state["players"].as_array() {
                for player in players {
                    if let Some(id) = player["id"].as_str() {
                        player_action_maps.insert(id.to_string(), json!({}));
                    }
                }
            }
            return player_action_maps;
        }

        let turn_player = state.get("currentPlayer")
            .and_then(|cp| cp.as_str())
            .unwrap_or("");
        println!("[DEBUG action_map] Current turn player: {}", turn_player);

        let players = state.get("players")
            .and_then(|p| p.as_array())
            .unwrap_or(&Vec::new())
            .clone();

        if !players.is_empty() {
            println!("[DEBUG action_map] Players in game: {:?}", players);
            for (idx, _player) in players.iter().enumerate() {
                let id = format!("p{}", idx + 1);
                println!("[DEBUG action_map] Checking actions for player: {}", id);
                let mut action_map = serde_json::Map::new();
                
                if id == turn_player {
                    println!("[DEBUG action_map] Player {} is the current turn player", id);
                    
                    // Get current phases - check phases at top level (our format)
                    let mut current_phases = Vec::new();
                    if let Some(phases) = state.get("phases").and_then(|p| p.as_object()) {
                        println!("[DEBUG action_map] Found phases: {:?}", phases);
                        for (_phase_set, phase_id) in phases {
                            if let Some(phase_str) = phase_id.as_str() {
                                // Extract just the phase ID (e.g., "play" from "game.play")
                                if let Some(phase_part) = phase_str.split('.').last() {
                                    current_phases.push(phase_part.to_string());
                                } else {
                                    current_phases.push(phase_str.to_string());
                                }
                            }
                        }
                    }
                    
                    println!("[DEBUG action_map] Current phases: {:?}", current_phases);
                    if let Some(actionlist) = bundle.actions.as_array() {
                        println!("[DEBUG action_map] Found {} actions in bundle, current phases: {:?}", actionlist.len(), current_phases);
                        
                        // Find which actions are allowed in current phases
                        if let Some(phase_sets) = bundle.phases.as_array() {
                            let mut allowed_actions = Vec::new();
                            
                            // For each current phase, we need to find it within the phase sets
                            for current_phase in &current_phases {
                                    // Phase sets contain nested phases
                                    for phase_set in phase_sets {
                                        if let Some(phases) = phase_set["phases"].as_array() {
                                            if let Some(phase_def) = phases.iter().find(|p| p["id"].as_str() == Some(current_phase)) {
                                                println!("[DEBUG action_map] Found phase {} with definition: {:?}", current_phase, phase_def);
                                                // Check both 'possibleActions' (new) and 'actions' (old) fields
                                                let actions = phase_def["possibleActions"].as_array()
                                                    .or_else(|| phase_def["actions"].as_array());
                                                
                                                if let Some(actions) = actions {
                                                    println!("[DEBUG action_map] Phase {} has {} possible actions", current_phase, actions.len());
                                                    for action in actions {
                                                        if let Some(action_id) = action.as_str() {
                                                            allowed_actions.push(action_id);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                
                                println!("[DEBUG action_map] Allowed actions in current phases: {:?}", allowed_actions);
                                
                                for a in actionlist {
                                    let action_id = a["id"].as_str().unwrap_or("");
                                    
                                    // Skip actions not allowed in current phases
                                    if !allowed_actions.contains(&action_id) {
                                        println!("[DEBUG action_map] Skipping action {} - not in allowed actions", action_id);
                                        continue;
                                    }
                                // Support both 'uses' (new) and 'builtin' (old)
                                let action_impl = a["uses"].as_str()
                                    .or_else(|| a["builtin"].as_str());
                                println!("[DEBUG action_map] Action {} has implementation: {:?}", a["id"].as_str().unwrap_or("unknown"), action_impl);
                                if action_impl == Some("place") {
                                    // Handle place action for tic-tac-toe and similar games
                                    println!("[DEBUG] Found place action: {:?}", a["id"]);
                                    if let Some(action_id) = a["id"].as_str() {
                                        // For place actions, we need to find all empty cells on the board
                                        if let Some(zones) = state.get("zones").and_then(|z| z.as_object()) {
                                            for (zone_id, zone_data) in zones {
                                                println!("[DEBUG] Checking zone {} for place action", zone_id);
                                                
                                                // Handle new format where zone has type and cells
                                                if let Some(zone_obj) = zone_data.as_object() {
                                                    if zone_obj.get("type").and_then(|t| t.as_str()) == Some("grid") {
                                                        if let Some(cells) = zone_obj.get("cells").and_then(|c| c.as_array()) {
                                                            println!("[DEBUG] Found grid zone with cells");
                                                            for (r, row) in cells.iter().enumerate() {
                                                                if let Some(row_array) = row.as_array() {
                                                                    for (c, cell) in row_array.iter().enumerate() {
                                                                        if cell.is_null() {
                                                                            let location = format!("/zones/{}/cells/{}/{}", zone_id, r, c);
                                                                            println!("[DEBUG] Empty cell at {}", location);
                                                                            
                                                                            // Get UI direction from action
                                                                            let direction = a.get("ui")
                                                                                .and_then(|ui| ui.get("direction"))
                                                                                .and_then(|d| d.as_str())
                                                                                .unwrap_or("Select this location");
                                                                            
                                                                            action_map.insert(location, serde_json::json!({
                                                                                "action": action_id,
                                                                                "direction": direction
                                                                            }));
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
                                } else if action_impl == Some("grid.move") || action_impl == Some("presets.grid.move") || action_impl == Some("moveEntity") {
                                    println!("[DEBUG] Found grid.move action: {:?}", a["id"]);
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(target_zone) = params["target"]["zone"].as_str() {
                                                println!("[DEBUG] Target zone: {}", target_zone);
                                            let zone_state = &state["zones"][target_zone];
                                            println!("[DEBUG] Zone state for {}: {:?}", target_zone, zone_state);
                                            if zone_state.is_array() {
                                                let mut valid_options = Vec::new();
                                                let gravity = params["target"]["gravity"].as_bool().unwrap_or(false);
                                                
                                                if gravity {
                                                    // For gravity mode, only check top row (row 0) and exclude full columns
                                                    if let Some(grid) = zone_state.as_array() {
                                                        let cols = grid.get(0).and_then(|r| r.as_array()).map(|r| r.len()).unwrap_or(0);
                                                        for c in 0..cols {
                                                            // Check if column has any empty cell
                                                            let mut has_empty = false;
                                                            for row in grid {
                                                                if let Some(cells) = row.as_array() {
                                                                    if cells.get(c).map(|cell| cell.is_null()).unwrap_or(false) {
                                                                        has_empty = true;
                                                                        break;
                                                                    }
                                                                }
                                                            }
                                                            if has_empty {
                                                                valid_options.push(serde_json::json!({
                                                                    "zone": target_zone,
                                                                    "row": 0, // Always top row for gravity
                                                                    "col": c
                                                                }));
                                                            }
                                                        }
                                                    }
                                                } else if action_impl == Some("grid.move") || action_impl == Some("moveEntity") {
                                                    println!("[DEBUG] Checking for flip effects");
                                                    // Check if this has flip effect (Reversi-style)
                                                    let has_flip_effect = params.get("effects").and_then(|e| e.as_array())
                                                        .map(|effects| effects.iter().any(|e| e["type"].as_str() == Some("flip")))
                                                        .unwrap_or(false);
                                                    
                                                    if has_flip_effect {
                                                        // Only show moves that would flip at least one piece
                                                        let source_template = params["source"].as_str().unwrap_or("");
                                                        let source_id = source_template.replace("{actor}", &id);
                                                        let player_piece = if let Some(z) = state["zones"].get(&source_id) {
                                                            z["infinite"].as_str().unwrap_or("")
                                                        } else {
                                                            ""
                                                        };
                                                        
                                                        if !player_piece.is_empty() {
                                                            for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                                for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                                    if cell.is_null() && would_flip_any(zone_state, r, c, player_piece) {
                                                                        valid_options.push(serde_json::json!({
                                                                            "zone": target_zone,
                                                                            "row": r,
                                                                            "col": c
                                                                        }));
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    } else {
                                                        // Normal moveEntity without special effects
                                                        println!("[DEBUG] Normal grid.move without flip effects");
                                                        for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                            for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                                if cell.is_null() {
                                                                    println!("[DEBUG] Found empty cell at row {} col {}", r, c);
                                                                    valid_options.push(serde_json::json!({
                                                                        "zone": target_zone,
                                                                        "row": r,
                                                                        "col": c
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // Normal mode - check all empty cells
                                                    for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                        for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                            if cell.is_null() {
                                                                valid_options.push(serde_json::json!({
                                                                    "zone": target_zone,
                                                                    "row": r,
                                                                    "col": c
                                                                }));
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                // Add each valid option to the action map
                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Make a move");
                                                for option in valid_options {
                                                    if let (Some(zone), Some(row), Some(col)) = 
                                                        (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                        let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                        action_map.insert(location, json!({
                                                            "action": action_id,
                                                            "direction": direction
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("grid.select") || action_impl == Some("selectEntity") {
                                    // Handle piece selection for checkers
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(zone_name) = params["zone"].as_str() {
                                            let zone_state = &state["zones"][zone_name];
                                            if zone_state.is_array() {
                                                let mut valid_options = Vec::new();
                                                
                                                // Find all of this player's pieces
                                                for (r, row) in zone_state.as_array().unwrap().iter().enumerate() {
                                                    for (c, cell) in row.as_array().unwrap().iter().enumerate() {
                                                        if let Some(entity) = cell.as_str() {
                                                            if entity.contains(&format!("_{}", id)) {
                                                                // This is the player's piece, check if it has valid moves
                                                                let is_king = entity.contains("king_");
                                                                let is_player1 = entity.contains("_p1");
                                                                let mut has_valid_move = false;
                                                                
                                                                // Check all possible moves for this piece
                                                                for dr in [-2i32, -1, 1, 2].iter() {
                                                                    for dc in [-2i32, -1, 1, 2].iter() {
                                                                        if dr.abs() != dc.abs() {
                                                                            continue; // Only diagonal moves
                                                                        }
                                                                        
                                                                        // Regular pieces can only move forward
                                                                        if !is_king {
                                                                            // Player 1 pieces move up (negative dr), Player 2 pieces move down (positive dr)
                                                                            if (is_player1 && *dr > 0) || (!is_player1 && *dr < 0) {
                                                                                continue; // Skip backward moves for regular pieces
                                                                            }
                                                                        }
                                                                        
                                                                        let new_row = r as i32 + dr;
                                                                        let new_col = c as i32 + dc;
                                                                        
                                                                        if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                                                                            // Check if target is empty
                                                                            if let Some(target_cell) = zone_state.as_array()
                                                                                .and_then(|z| z.get(new_row as usize))
                                                                                .and_then(|r| r.as_array())
                                                                                .and_then(|r| r.get(new_col as usize)) {
                                                                                
                                                                                if target_cell.is_null() {
                                                                                    has_valid_move = true;
                                                                                    break;
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    if has_valid_move {
                                                                        break;
                                                                    }
                                                                }
                                                                
                                                                if has_valid_move {
                                                                    valid_options.push(serde_json::json!({
                                                                        "zone": zone_name,
                                                                        "row": r,
                                                                        "col": c
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                
                                                // Add each valid option to the action map
                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Select piece");
                                                for option in valid_options {
                                                    if let (Some(zone), Some(row), Some(col)) = 
                                                        (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                        let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                        action_map.insert(location, json!({
                                                            "action": action_id,
                                                            "direction": direction
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("grid.moveSelected") || action_impl == Some("moveSelected") {
                                    // Handle moving the selected piece (checkers)
                                    if let Some(action_id) = a["id"].as_str() {
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(target_zone) = params["zone"].as_str() {
                                            if let Some(selected) = state.get("selections").and_then(|s| s.get(&id)) {
                                                let source_zone = selected["zone"].as_str().unwrap_or("");
                                                let source_row = selected["row"].as_u64().unwrap_or(0) as usize;
                                                let source_col = selected["col"].as_u64().unwrap_or(0) as usize;
                                                
                                                // Get the selected piece entity
                                                let zone_state = &state["zones"][source_zone];
                                                let entity = zone_state[source_row][source_col].as_str().unwrap_or("");
                                                let is_king = entity.contains("king_");
                                                let is_player1 = entity.contains("_p1");
                                                
                                                let target_state = &state["zones"][target_zone];
                                                if target_state.is_array() {
                                                    // Check for mandatory captures first
                                                    let (has_capture, _) = if let Some(zones) = state["zones"].as_object() {
                                                        let mut has_any_capture = false;
                                                        let mut capture_map = std::collections::HashMap::new();
                                                        
                                                        // Check all pieces for capture opportunities
                                                        for (zone_id, zone) in zones {
                                                            if let Some(cells) = zone.as_array() {
                                                                for (r, row) in cells.iter().enumerate() {
                                                                    if let Some(row_array) = row.as_array() {
                                                                        for (c, cell) in row_array.iter().enumerate() {
                                                                            if let Some(piece) = cell.as_str() {
                                                                                if piece.contains(&format!("_{}", id)) {
                                                                                    // Check if this piece can capture
                                                                                    let piece_is_king = piece.contains("king_");
                                                                                    let piece_is_player1 = piece.contains("_p1");
                                                                                    
                                                                                    for dr in [-2i32, 2].iter() {
                                                                                        for dc in [-2i32, 2].iter() {
                                                                                            // Regular pieces can only capture forward
                                                                                            if !piece_is_king {
                                                                                                if (piece_is_player1 && *dr > 0) || (!piece_is_player1 && *dr < 0) {
                                                                                                    continue;
                                                                                                }
                                                                                            }
                                                                                            
                                                                                            let jump_row = r as i32 + dr;
                                                                                            let jump_col = c as i32 + dc;
                                                                                            let mid_row = r as i32 + dr / 2;
                                                                                            let mid_col = c as i32 + dc / 2;
                                                                                            
                                                                                            if jump_row >= 0 && jump_row < 8 && jump_col >= 0 && jump_col < 8 &&
                                                                                               mid_row >= 0 && mid_row < 8 && mid_col >= 0 && mid_col < 8 {
                                                                                                
                                                                                                if let Some(target_cell) = cells
                                                                                                    .get(jump_row as usize)
                                                                                                    .and_then(|r| r.as_array())
                                                                                                    .and_then(|r| r.get(jump_col as usize)) {
                                                                                                    
                                                                                                    if target_cell.is_null() {
                                                                                                        if let Some(mid_cell) = cells
                                                                                                            .get(mid_row as usize)
                                                                                                            .and_then(|r| r.as_array())
                                                                                                            .and_then(|r| r.get(mid_col as usize))
                                                                                                            .and_then(|c| c.as_str()) {
                                                                                                            
                                                                                                            if !mid_cell.contains(&format!("_{}", id)) && !mid_cell.is_empty() {
                                                                                                                has_any_capture = true;
                                                                                                                if zone_id == source_zone && r == source_row && c == source_col {
                                                                                                                    capture_map.insert((jump_row as usize, jump_col as usize), true);
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
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        (has_any_capture, capture_map)
                                                    } else {
                                                        (false, std::collections::HashMap::new())
                                                    };
                                                    
                                                    let must_capture_only = is_king;
                                                    
                                                    let mut capture_moves = Vec::new();
                                                    let mut regular_moves = Vec::new();
                                                    
                                                    // Calculate valid moves (diagonals only for checkers)
                                                    for dr in [-2i32, -1, 1, 2].iter() {
                                                        for dc in [-2i32, -1, 1, 2].iter() {
                                                            if dr.abs() != dc.abs() {
                                                                continue; // Only diagonal moves
                                                            }
                                                            
                                                            // Regular pieces can only move forward
                                                            if !is_king {
                                                                // Player 1 pieces move up (negative dr), Player 2 pieces move down (positive dr)
                                                                if (is_player1 && *dr > 0) || (!is_player1 && *dr < 0) {
                                                                    continue; // Skip backward moves for regular pieces
                                                                }
                                                            }
                                                            
                                                            let new_row = source_row as i32 + dr;
                                                            let new_col = source_col as i32 + dc;
                                                            
                                                            if new_row >= 0 && new_row < 8 && new_col >= 0 && new_col < 8 {
                                                                let target_row = new_row as usize;
                                                                let target_col = new_col as usize;
                                                                
                                                                // Check if target is empty
                                                                if let Some(target_cell) = zone_state.as_array()
                                                                    .and_then(|z| z.get(target_row))
                                                                    .and_then(|r| r.as_array())
                                                                    .and_then(|r| r.get(target_col)) {
                                                                    
                                                                    if target_cell.is_null() {
                                                                        // For captures, check if there's an opponent piece to jump
                                                                        if dr.abs() == 2 {
                                                                            let mid_row = ((source_row as i32 + new_row) / 2) as usize;
                                                                            let mid_col = ((source_col as i32 + new_col) / 2) as usize;
                                                                            
                                                                            if let Some(mid_cell) = zone_state.as_array()
                                                                                .and_then(|z| z.get(mid_row))
                                                                                .and_then(|r| r.as_array())
                                                                                .and_then(|r| r.get(mid_col))
                                                                                .and_then(|c| c.as_str()) {
                                                                                
                                                                                // Check if it's opponent's piece
                                                                                if !mid_cell.contains(&format!("_{}", id)) && !mid_cell.is_empty() {
                                                                                    capture_moves.push(serde_json::json!({
                                                                                        "zone": target_zone,
                                                                                        "row": target_row,
                                                                                        "col": target_col
                                                                                    }));
                                                                                }
                                                                            }
                                                                        } else {
                                                                            // Regular move
                                                                            regular_moves.push(serde_json::json!({
                                                                                "zone": target_zone,
                                                                                "row": target_row,
                                                                                "col": target_col
                                                                            }));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    
                                                    // If this is a king that can capture, only show capture moves
                                                    if must_capture_only && !capture_moves.is_empty() {
                                                        valid_options = capture_moves;
                                                    } else if !capture_moves.is_empty() || !regular_moves.is_empty() {
                                                        valid_options = capture_moves;
                                                        valid_options.extend(regular_moves);
                                                    }
                                                    
                                                    // Add each valid option to the action map
                                                    let direction = a["ui"]["direction"].as_str().unwrap_or("Move piece");
                                                    for option in valid_options {
                                                        if let (Some(zone), Some(row), Some(col)) = 
                                                            (option["zone"].as_str(), option["row"].as_u64(), option["col"].as_u64()) {
                                                            let location = format!("/zones/{}/{}/{}", zone, row, col);
                                                            action_map.insert(location, json!({
                                                                "action": action_id,
                                                                "direction": direction
                                                            }));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("deck.draw") || action_impl == Some("drawCard") {
                                    if let Some(action_id) = a["id"].as_str() {
                                        // For drawCard, check if source deck has cards
                                        // Support both 'with' (new) and 'params' (old)
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(source) = params["source"].as_str() {
                                            if let Some(deck) = state["zones"][source].get("items").and_then(|i| i.as_array()) {
                                                if !deck.is_empty() {
                                                    let direction = a["ui"]["direction"].as_str().unwrap_or("Draw a card");
                                                    // For non-grid zones, use the zone itself as the location
                                                    let location = format!("/zones/{}", source);
                                                    action_map.insert(location, json!({
                                                        "action": action_id,
                                                        "direction": direction
                                                    }));
                                                }
                                            }
                                        }
                                    }
                                    }
                                } else if action_impl == Some("entity.move") || action_impl == Some("presets.entity.move") {
                                    if let Some(action_id) = a["id"].as_str() {
                                        println!("[DEBUG action_map] Processing entity.move action: {}", action_id);
                                        // Handle entity.move actions for card games
                                        let params = a.get("with").or_else(|| a.get("params"));
                                        if let Some(params) = params {
                                            if let Some(source) = params["source"].as_str() {
                                                let source_zone = source.replace("{actor}", &id);
                                                println!("[DEBUG action_map] Source zone: {} (from {})", source_zone, source);
                                                
                                                // Check if this is a zone-level action (drawing from deck/discard)
                                                if source_zone == "drawPile" || source_zone == "discardPile" {
                                                    if let Some(zone_data) = state["zones"][&source_zone].as_object() {
                                                        if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
                                                            println!("[DEBUG action_map] Zone {} has {} items", source_zone, items.len());
                                                            if !items.is_empty() {
                                                                // Check conditions before adding action
                                                                let mut conditions_met = true;
                                                                if let Some(conditions) = a.get("conditions").and_then(|c| c.as_array()) {
                                                                    for condition in conditions {
                                                                        if let Some(cond_type) = condition.get("type").and_then(|t| t.as_str()) {
                                                                            if cond_type == "zone.count" {
                                                                                if let Some(with) = condition.get("with") {
                                                                                    let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", &id);
                                                                                    // Get the zone we're checking
                                                                                    if let Some(check_zone) = state["zones"].get(&zone_id) {
                                                                                        if let Some(check_items) = check_zone.get("items").and_then(|i| i.as_array()) {
                                                                                            let count = check_items.len();
                                                                                            if let Some(exact) = with.get("exact").and_then(|e| e.as_u64()) {
                                                                                                println!("[DEBUG conditions] Checking zone.count exact: {} == {}", count, exact);
                                                                                                if count != exact as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                            if let Some(min) = with.get("min").and_then(|m| m.as_u64()) {
                                                                                                if count < min as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                            if let Some(max) = with.get("max").and_then(|m| m.as_u64()) {
                                                                                                if count > max as usize {
                                                                                                    conditions_met = false;
                                                                                                    break;
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                
                                                                if conditions_met {
                                                                    let direction = a["ui"]["direction"].as_str().unwrap_or("Select");
                                                                    let location = format!("/zones/{}", source_zone);
                                                                    println!("[DEBUG action_map] Adding zone action at {} -> {}", location, action_id);
                                                                    action_map.insert(location, json!({
                                                                        "action": action_id,
                                                                        "direction": direction
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else if source_zone.starts_with("hand_") {
                                                    // This is for discarding cards from hand
                                                    if let Some(zone_data) = state["zones"][&source_zone].as_object() {
                                                        if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
                                                            println!("[DEBUG action_map] Hand {} has {} items", source_zone, items.len());
                                                            println!("[DEBUG action_map] Processing action {} for hand", action_id);
                                                            
                                                            // Check conditions before adding action
                                                            let mut conditions_met = true;
                                                            if let Some(conditions) = a.get("conditions").and_then(|c| c.as_array()) {
                                                                for condition in conditions {
                                                                    if let Some(cond_type) = condition.get("type").and_then(|t| t.as_str()) {
                                                                        if cond_type == "zone.count" {
                                                                            if let Some(with) = condition.get("with") {
                                                                                let zone_id = with.get("zone").and_then(|z| z.as_str()).unwrap_or("").replace("{actor}", &id);
                                                                                if zone_id == source_zone {
                                                                                    if let Some(exact) = with.get("exact").and_then(|e| e.as_u64()) {
                                                                                        println!("[DEBUG conditions] Checking exact count for {}: {} == {}", zone_id, items.len(), exact);
                                                                                        if items.len() != exact as usize {
                                                                                            println!("[DEBUG conditions] Condition failed: {} != {}", items.len(), exact);
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    if let Some(min) = with.get("min").and_then(|m| m.as_u64()) {
                                                                                        if items.len() < min as usize {
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                    if let Some(max) = with.get("max").and_then(|m| m.as_u64()) {
                                                                                        if items.len() > max as usize {
                                                                                            conditions_met = false;
                                                                                            break;
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            
                                                            if conditions_met {
                                                                let direction = a["ui"]["direction"].as_str().unwrap_or("Select card");
                                                                // Add action for each card in hand
                                                                for (index, _card) in items.iter().enumerate() {
                                                                    let location = format!("/zones/{}/{}", source_zone, index);
                                                                    action_map.insert(location, json!({
                                                                        "action": action_id,
                                                                        "direction": direction
                                                                    }));
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } // Close phases_array check
                        }
                    }
                    
                    // Insert the action map for this player
                    player_action_maps.insert(id.to_string(), json!(action_map));
                }
            }
        }

        player_action_maps
    }