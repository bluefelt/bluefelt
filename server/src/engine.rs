use crate::bundle::Bundle;
use serde_json::{json, Value, Map};

/* --------------------------------------------------------------------------
   Load initial state from bundle
   ----------------------------------------------------------------------- */
pub fn load_initial_state(bundle: &Bundle) -> Value {
    let player_count = bundle
        .manifest
        .metadata
        .players
        .max;
    let mut players = Vec::new();
    for i in 1..=player_count {
        players.push(json!({"id": format!("p{}", i)}));
    }

    let mut zones = Map::new();
    if let Some(arr) = bundle.zones.as_array() {
        for zone in arr {
            let id = zone["id"].as_str().unwrap_or("");
            let shape = zone["shape"].as_str().unwrap_or("");
            let contents = zone
                .get("contents")
                .cloned()
                .unwrap_or_else(|| Value::String("empty".to_string()));
            let per_player = id.contains("{player}");
            let ids: Vec<String> = if per_player {
                players
                    .iter()
                    .map(|p| id.replace("{player}", p["id"].as_str().unwrap()))
                    .collect()
            } else {
                vec![id.to_string()]
            };

            for pid in ids {
                let mut content_spec = contents.clone();
                if per_player {
                    let player_id = pid.split('_').last().unwrap_or("");
                    if let Some(map) = contents.as_object() {
                        if let Some(specific) = map.get(player_id) {
                            content_spec = specific.clone();
                        }
                    }
                }

                let value = match shape {
                    "grid" => init_grid(zone, &content_spec),
                    "list" => init_list(&content_spec),
                    _ => Value::Null,
                };
                zones.insert(pid, value);
            }
        }
    }

    json!({
        "zones": Value::Object(zones),
        "players": players,
        "turn": "p1"
    })
}

fn init_grid(zone: &Value, contents: &Value) -> Value {
    let width = zone
        .get("gridProps")
        .and_then(|g| g.get("width"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let height = zone
        .get("gridProps")
        .and_then(|g| g.get("height"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if contents.as_str() == Some("empty") {
        let mut rows = Vec::new();
        for _ in 0..height {
            rows.push(vec![Value::Null; width]);
        }
        return Value::Array(rows.into_iter().map(Value::Array).collect());
    }
    if let Some(arr) = contents.as_array() {
        let mut rows: Vec<Value> = Vec::new();
        for row in arr {
            if let Some(cells) = row.as_array() {
                // wrap the row so the outer Vec is Vec<Value>
                rows.push(Value::Array(cells.clone()));
            }
        }
        return Value::Array(rows);
    }
    Value::Null
}

fn init_list(contents: &Value) -> Value {
    if contents.as_str() == Some("empty") {
        return json!({"items": []});
    }
    if let Some(arr) = contents.as_array() {
        return json!({"items": arr.clone()});
    }
    if let Some(obj) = contents.as_object() {
        let entity = obj.get("entity").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(count) = obj.get("count") {
            if count.as_str() == Some("infinite") {
                return json!({"items": [], "infinite": entity});
            } else if let Some(n) = count.as_u64() {
                let mut items = Vec::new();
                for _ in 0..n {
                    items.push(Value::String(entity.to_string()));
                }
                return json!({"items": items});
            }
        }
    }
    json!({"items": []})
}

/* --------------------------------------------------------------------------
   Apply verb
   ----------------------------------------------------------------------- */
pub fn apply_verb(
    bundle: &Bundle,
    state: &mut Value,
    caller: &str,
    action: &Value,
) -> Value {
    let verb_id = action["verb"].as_str().unwrap_or("");
    let verb_spec = bundle
        .verbs
        .as_array()
        .and_then(|v| v.iter().find(|x| x["id"].as_str() == Some(verb_id)));
    let Some(spec) = verb_spec else { return json!([]) };
    
    // Check if this is an auto verb (no turn validation)
    let is_auto = spec["auto"].as_bool().unwrap_or(false);
    
    // Don't allow any non-auto verbs if game has ended
    if !is_auto {
        if let Some(meta) = state.get("meta") {
            if let Some(game_status) = meta.get("gameStatus") {
                if game_status["state"].as_str() == Some("ended") {
                    return json!([]);
                }
            }
        }
        
        // Ensure the caller is the active player
        if state["turn"].as_str() != Some(caller) {
            return json!([]);
        }
    }

    let mut patches = vec![];
    
    // Apply the verb
    let verb_patches = if spec["builtin"].as_str() == Some("moveEntity") {
        apply_move_entity(bundle, state, spec, action, caller)
    } else if spec["builtin"].as_str() == Some("nextTurn") {
        apply_next_turn(state, caller)
    } else if spec.get("hook").is_some() {
        apply_hook(bundle, state, spec, action, caller)
    } else if spec.get("conditions").is_some() {
        apply_conditions(bundle, state, spec, action, caller)
    } else {
        json!([])
    };
    
    if let Some(arr) = verb_patches.as_array() {
        patches.extend_from_slice(arr);
    }
    
    // Apply patches to state before triggers
    for patch in &patches {
        apply_patch_to_state(state, patch);
    }
    
    // Process triggers
    if let Some(triggers) = spec["triggers"].as_array() {
        for trigger in triggers {
            if let Some(trigger_id) = trigger.as_str() {
                // Create a trigger action
                let trigger_action = json!({ "verb": trigger_id });
                let trigger_patches = apply_verb(bundle, state, caller, &trigger_action);
                if let Some(arr) = trigger_patches.as_array() {
                    patches.extend_from_slice(arr);
                    // Apply trigger patches to state as well
                    for patch in arr {
                        apply_patch_to_state(state, patch);
                    }
                }
            }
        }
    }
    
    // Remove turn advancement patches if game has ended
    if let Some(meta) = state.get("meta") {
        if let Some(game_status) = meta.get("gameStatus") {
            if game_status["state"].as_str() == Some("ended") {
                patches.retain(|p| p["path"].as_str() != Some("/turn"));
            }
        }
    }
    
    json!(patches)
}

fn apply_move_entity(
    _bundle: &Bundle,
    state: &mut Value,
    spec: &Value,
    action: &Value,
    actor: &str,
) -> Value {
    let source_template = spec["params"]["source"].as_str().unwrap_or("");
    let source_id = source_template.replace("{actor}", actor);
    let target_zone = spec["params"]["target"]["zone"].as_str().unwrap_or("");
    let target_id = target_zone.replace("{actor}", actor);

    // Extract entity from source
    let mut entity = String::new();
    let mut remove_source = false;
    if let Some(z) = state["zones"].get_mut(&source_id) {
        if z.get("infinite").is_some() {
            entity = z["infinite"].as_str().unwrap().to_string();
        } else if let Some(items) = z.get_mut("items").and_then(|v| v.as_array_mut()) {
            if let Some(val) = items.first() {
                entity = val.as_str().unwrap_or("").to_string();
                items.remove(0);
                remove_source = true;
            }
        }
    }
    if entity.is_empty() {
        return json!([]);
    }
    

    // Apply to target
    let mut ops = Vec::new();
    if let Some(zone_val) = state["zones"].get_mut(&target_id) {
        if zone_val.is_array() {
            // grid
            let mut row = action["args"]["row"].as_u64().unwrap_or(0) as usize;
            let col = action["args"]["col"].as_u64().unwrap_or(0) as usize;
            
            // Check for gravity mode
            let gravity = spec["params"]["target"]["gravity"].as_bool().unwrap_or(false);
            
            if gravity {
                // For gravity mode, find the lowest empty row in the column
                if let Some(grid) = zone_val.as_array() {
                    // First check if column is full
                    let mut column_full = true;
                    for r in 0..grid.len() {
                        if let Some(row_arr) = grid[r].as_array() {
                            if row_arr.get(col).map(|c| c.is_null()).unwrap_or(false) {
                                column_full = false;
                                break;
                            }
                        }
                    }
                    
                    if column_full {
                        return json!([]); // Column is full, can't place
                    }
                    
                    // Find lowest empty row
                    for r in (0..grid.len()).rev() {
                        if let Some(row_arr) = grid[r].as_array() {
                            if row_arr.get(col).map(|c| c.is_null()).unwrap_or(false) {
                                row = r;
                                break;
                            }
                        }
                    }
                }
            }
            
            if let Some(row_arr) = zone_val.as_array_mut().and_then(|r| r.get_mut(row)) {
                if let Some(cells) = row_arr.as_array_mut() {
                    if cells[col].is_null() {
                        cells[col] = Value::String(entity.clone());
                        ops.push(json!({
                            "op": "replace",
                            "path": format!("/zones/{}/{}/{}", target_id, row, col),
                            "value": entity
                        }));
                    } else {
                        return json!([]);
                    }
                }
            }
        } else if zone_val.get("items").is_some() {
            let items = zone_val.get_mut("items").unwrap().as_array_mut().unwrap();
            let _idx = items.len();
            items.push(Value::String(entity.clone()));
            ops.push(json!({
                "op": "add",
                "path": format!("/zones/{}/items/-", target_id),
                "value": entity
            }));
        }
    }

    if remove_source {
        ops.insert(0, json!({
            "op": "remove",
            "path": format!("/zones/{}/items/0", source_id)
        }));
    }

    Value::Array(ops)
}

fn apply_next_turn(state: &mut Value, caller: &str) -> Value {
    // Get next player in turn order
    let next_turn = {
        if let Some(players) = state["players"].as_array() {
            players
                .iter()
                .position(|p| p["id"].as_str() == Some(caller))
                .map(|idx| {
                    players[(idx + 1) % players.len()]["id"]
                        .as_str()
                        .unwrap()
                        .to_string()
                })
        } else {
            None
        }
    };

    if let Some(next) = next_turn {
        state["turn"] = Value::String(next.clone());
        return json!([{ "op": "replace", "path": "/turn", "value": next }]);
    }
    
    json!([])
}

/* --------------------------------------------------------------------------
   Check tic-tac-toe game end
   ----------------------------------------------------------------------- */
fn apply_patch_to_state(state: &mut Value, patch: &Value) {
    // Simple patch application for our use case
    if let (Some(op), Some(path), Some(value)) = (
        patch["op"].as_str(),
        patch["path"].as_str(),
        patch.get("value")
    ) {
        if op == "add" || op == "replace" {
            // Parse the path and apply the change
            let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
            let mut current = state;
            
            for (i, part) in parts.iter().enumerate() {
                if i == parts.len() - 1 {
                    // Last part - set the value
                    if let Some(obj) = current.as_object_mut() {
                        obj.insert(part.to_string(), value.clone());
                    }
                } else {
                    // Navigate deeper - create missing objects
                    if !current.get(part).is_some() {
                        if let Some(obj) = current.as_object_mut() {
                            obj.insert(part.to_string(), json!({}));
                        }
                    }
                    // Safety check before unwrap
                    if let Some(next) = current.get_mut(part) {
                        current = next;
                    } else {
                        // Path doesn't exist and we couldn't create it
                        return;
                    }
                }
            }
        }
    }
}

fn apply_conditions(_bundle: &Bundle, state: &Value, spec: &Value, _action: &Value, caller: &str) -> Value {
    println!("[DEBUG] apply_conditions called for verb: {:?}, caller: {}", spec["id"], caller);
    
    let conditions = match spec["conditions"].as_array() {
        Some(c) => c,
        None => return json!([]),
    };
    
    let mut patches = vec![];
    
    // For checkWin or checkGameEnd, we need to check conditions for all players
    let players_to_check = if spec["id"].as_str() == Some("checkGameEnd") || spec["id"].as_str() == Some("checkWin") {
        // Get all players from state
        if let Some(players) = state["players"].as_array() {
            players.iter()
                .filter_map(|p| p["id"].as_str())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        } else {
            vec![caller.to_string()]
        }
    } else {
        vec![caller.to_string()]
    };
    
    for check_player in &players_to_check {
        for condition in conditions {
            let condition_type = match condition["type"].as_str() {
                Some(t) => t,
                None => continue,
            };
            
            let condition_met = match condition_type {
                "consecutiveMarksInRow" => check_consecutive_marks(state, condition, check_player),
                "allCellsFilled" => check_all_cells_filled(state, condition),
                "any" => {
                    // Handle nested conditions for "any" type
                    if let Some(sub_conditions) = condition["conditions"].as_array() {
                        for sub_cond in sub_conditions {
                            let sub_met = match sub_cond["type"].as_str() {
                                Some("consecutiveMarksInRow") => check_consecutive_marks(state, sub_cond, check_player),
                                Some("allCellsFilled") => check_all_cells_filled(state, sub_cond),
                                _ => false,
                            };
                            
                            if sub_met {
                                // Apply the result from the sub-condition
                                let result = sub_cond["result"].as_str().unwrap_or("");
                                match result {
                                    "gameWin" => {
                                        patches.push(json!({
                                            "op": "add",
                                            "path": "/meta/gameStatus",
                                            "value": {
                                                "state": "ended",
                                                "winner": check_player,
                                                "tie": false
                                            }
                                        }));
                                        return json!(patches);
                                    }
                                    "gameTie" => {
                                        patches.push(json!({
                                            "op": "add",
                                            "path": "/meta/gameStatus",
                                            "value": {
                                                "state": "ended",
                                                "tie": true
                                            }
                                        }));
                                        return json!(patches);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        false
                    } else {
                        false
                    }
                },
                _ => false,
            };
            
            if condition_met {
                // Apply the result
                let result = condition["result"].as_str().unwrap_or("");
                match result {
                    "gameWin" => {
                        patches.push(json!({
                            "op": "add",
                            "path": "/meta/gameStatus",
                            "value": {
                                "state": "ended",
                                "winner": check_player,
                                "tie": false
                            }
                        }));
                        return json!(patches); // Exit after finding a winner
                    }
                    "gameTie" => {
                        patches.push(json!({
                            "op": "add",
                            "path": "/meta/gameStatus",
                            "value": {
                                "state": "ended",
                                "tie": true
                            }
                        }));
                        return json!(patches);
                    }
                    _ => {}
                }
            }
        }
    }
    
    json!(patches)
}

fn check_consecutive_marks(state: &Value, condition: &Value, caller: &str) -> bool {
    let params = &condition["params"];
    let zone_name = params["zone"].as_str().unwrap_or("");
    let count = params["count"].as_u64().unwrap_or(3) as usize;
    let entity_template = params["entity"].as_str().unwrap_or("");
    let entity = entity_template.replace("{actor}", caller);
    
    println!("[DEBUG] Checking consecutive marks - zone: {}, count: {}, entity: {}, caller: {}", 
        zone_name, count, entity, caller);
    
    let board = match state["zones"][zone_name].as_array() {
        Some(b) => b,
        None => {
            println!("[DEBUG] Zone {} not found or not an array", zone_name);
            return false;
        }
    };
    
    let rows = board.len();
    if rows == 0 {
        return false;
    }
    let cols = board[0].as_array().map(|r| r.len()).unwrap_or(0);
    
    // Check rows
    for (row_idx, row) in board.iter().enumerate() {
        if let Some(cells) = row.as_array() {
            println!("[DEBUG] Row {}: {:?}", row_idx, cells);
            for window in cells.windows(count) {
                if window.iter().all(|cell| cell.as_str() == Some(&entity)) {
                    println!("[DEBUG] Found {} consecutive {} in row {}!", count, entity, row_idx);
                    return true;
                }
            }
        }
    }
    
    // Check columns
    for col in 0..cols {
        let mut consecutive = 0;
        for row in 0..rows {
            if let Some(cell) = board[row].as_array().and_then(|r| r.get(col)) {
                if cell.as_str() == Some(&entity) {
                    consecutive += 1;
                    if consecutive >= count {
                        return true;
                    }
                } else {
                    consecutive = 0;
                }
            }
        }
    }
    
    // Check diagonals
    // Top-left to bottom-right
    for start_row in 0..=rows.saturating_sub(count) {
        for start_col in 0..=cols.saturating_sub(count) {
            let mut all_match = true;
            for i in 0..count {
                let row = start_row + i;
                let col = start_col + i;
                if let Some(cell) = board.get(row)
                    .and_then(|r| r.as_array())
                    .and_then(|r| r.get(col)) {
                    if cell.as_str() != Some(&entity) {
                        all_match = false;
                        break;
                    }
                } else {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                return true;
            }
        }
    }
    
    // Top-right to bottom-left
    for start_row in 0..=rows.saturating_sub(count) {
        for start_col in (count - 1)..cols {
            let mut all_match = true;
            for i in 0..count {
                let row = start_row + i;
                let col = start_col - i;
                if let Some(cell) = board.get(row)
                    .and_then(|r| r.as_array())
                    .and_then(|r| r.get(col)) {
                    if cell.as_str() != Some(&entity) {
                        all_match = false;
                        break;
                    }
                } else {
                    all_match = false;
                    break;
                }
            }
            if all_match {
                return true;
            }
        }
    }
    
    false
}

fn check_all_cells_filled(state: &Value, condition: &Value) -> bool {
    let params = &condition["params"];
    let zone_name = params["zone"].as_str().unwrap_or("");
    
    let board = match state["zones"][zone_name].as_array() {
        Some(b) => b,
        None => return false,
    };
    
    board.iter().all(|row| {
        row.as_array()
            .map(|r| r.iter().all(|cell| !cell.is_null()))
            .unwrap_or(false)
    })
}

fn apply_hook(_bundle: &Bundle, state: &mut Value, spec: &Value, _action: &Value, _caller: &str) -> Value {
    let hook_name = match spec["hook"].as_str() {
        Some(name) => name,
        None => return json!([]),
    };
    
    // For now, implement checkGameEnd directly
    // TODO: In production, use HookRuntime to execute WASM modules
    if hook_name == "checkGameEnd" {
        return check_tic_tac_toe_game_end(state);
    }
    
    json!([])
}

fn check_tic_tac_toe_game_end(state: &Value) -> Value {
    let board = match state["zones"]["board"].as_array() {
        Some(b) => b,
        None => return json!([]),
    };

    // Check for winner
    if let Some(winner) = check_winner(board) {
        return json!([{
            "op": "add",
            "path": "/meta/gameStatus",
            "value": {
                "state": "ended",
                "winner": winner,
                "tie": false
            }
        }]);
    }

    // Check for tie (board full)
    let is_full = board.iter().all(|row| {
        row.as_array()
            .map(|r| r.iter().all(|cell| !cell.is_null()))
            .unwrap_or(false)
    });

    if is_full {
        return json!([{
            "op": "add",
            "path": "/meta/gameStatus",
            "value": {
                "state": "ended",
                "tie": true
            }
        }]);
    }

    json!([])
}

fn check_winner(board: &[Value]) -> Option<String> {
    // Convert board to a more manageable format
    let mut cells: Vec<Vec<Option<String>>> = vec![vec![None; 3]; 3];
    for (r, row) in board.iter().enumerate() {
        if let Some(row_array) = row.as_array() {
            for (c, cell) in row_array.iter().enumerate() {
                if r < 3 && c < 3 {
                    cells[r][c] = cell.as_str().map(|s| s.to_string());
                }
            }
        }
    }

    // Check rows
    for r in 0..3 {
        if let Some(winner) = check_line(&cells[r][0], &cells[r][1], &cells[r][2]) {
            return Some(winner);
        }
    }

    // Check columns
    for col in 0..3 {
        if let Some(winner) = check_line(&cells[0][col], &cells[1][col], &cells[2][col]) {
            return Some(winner);
        }
    }

    // Check diagonals
    if let Some(winner) = check_line(&cells[0][0], &cells[1][1], &cells[2][2]) {
        return Some(winner);
    }
    if let Some(winner) = check_line(&cells[0][2], &cells[1][1], &cells[2][0]) {
        return Some(winner);
    }

    None
}

fn check_line(a: &Option<String>, b: &Option<String>, c: &Option<String>) -> Option<String> {
    match (a, b, c) {
        (Some(mark_a), Some(mark_b), Some(mark_c)) if mark_a == mark_b && mark_b == mark_c => {
            // Convert mark to player ID (mark_x -> p1, mark_o -> p2)
            if mark_a == "mark_x" {
                Some("p1".to_string())
            } else if mark_a == "mark_o" {
                Some("p2".to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}
