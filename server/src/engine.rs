use crate::bundle::Bundle;
use serde_json::{json, Value, Map};
use rand::seq::SliceRandom;
use rand::rng;

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
            // Support both 'type' (new) and 'shape' (old) 
            let zone_type = zone["type"].as_str()
                .or_else(|| zone["shape"].as_str())
                .unwrap_or("");
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

                let mut value = match zone_type {
                    "grid" => init_grid(zone, &content_spec),
                    "list" | "deck" => init_list(&content_spec),
                    _ => Value::Null,
                };
                
                // Apply deck shuffling if specified
                if zone_type == "deck" {
                    if let Some(deck_props) = zone.get("deckProps") {
                        if deck_props.get("shuffle").and_then(|s| s.as_bool()).unwrap_or(false) {
                            if let Some(items) = value.get_mut("items").and_then(|i| i.as_array_mut()) {
                                let mut rng = rng();
                                items.shuffle(&mut rng);
                                println!("  Shuffled deck for zone: {}", pid);
                            }
                        }
                    }
                }
                
                zones.insert(pid, value);
            }
        }
    }

    // Get initial phase from manifest setup section
    let initial_phase = bundle.manifest.setup
        .as_ref()
        .and_then(|s| s.get("initialPhase"))
        .and_then(|p| p.as_str())
        .unwrap_or("play"); // Default to "play" if not specified
    
    json!({
        "zones": Value::Object(zones),
        "players": players,
        "turn": "p1",
        "meta": {
            "currentPhase": initial_phase,
            "gameStatus": {
                "state": "active"
            }
        }
    })
}

fn init_grid(zone: &Value, contents: &Value) -> Value {
    // Support both new (rows/cols) and old (width/height) naming
    let grid_props = zone.get("gridProps");
    let cols = grid_props
        .and_then(|g| g.get("cols").or_else(|| g.get("width")))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    let rows = grid_props
        .and_then(|g| g.get("rows").or_else(|| g.get("height")))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;
    if contents.as_str() == Some("empty") {
        let mut grid_rows = Vec::new();
        for _ in 0..rows {
            grid_rows.push(vec![Value::Null; cols]);
        }
        return Value::Array(grid_rows.into_iter().map(Value::Array).collect());
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
   Apply action
   ----------------------------------------------------------------------- */
pub fn apply_action(
    bundle: &Bundle,
    state: &mut Value,
    caller: &str,
    action: &Value,
) -> Value {
    let action_id = action["action"].as_str().unwrap_or("");
    println!("[DEBUG apply_action] Looking for action: {}", action_id);
    
    // First try to find the action in the bundle
    let action_spec = bundle
        .actions
        .as_array()
        .and_then(|a| a.iter().find(|x| x["id"].as_str() == Some(action_id)));
    
    // If not found and it's a builtin action, create a minimal spec
    let builtin_spec;
    let spec = if let Some(s) = action_spec {
        println!("[DEBUG apply_action] Found action {} in bundle", action_id);
        s
    } else if action_id.contains('.') {
        // This looks like a builtin action (e.g., "phase.set", "entity.move")
        println!("[DEBUG apply_action] Creating builtin spec for {}", action_id);
        builtin_spec = json!({
            "id": action_id,
            "uses": action_id,
            "auto": true,
            "with": action.get("with").cloned().unwrap_or(json!({}))
        });
        &builtin_spec
    } else {
        println!("[DEBUG apply_action] Action {} not found!", action_id);
        return json!([]);
    };
    
    // Check if this is an auto action (no turn validation)
    let is_auto = spec["auto"].as_bool().unwrap_or(false);
    
    // Don't allow any non-auto actions if game has ended
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
    
    // Check 'when' conditions if present
    if let Some(when_conditions) = spec.get("when").and_then(|w| w.as_array()) {
        for condition in when_conditions {
            if let Some(cond_type) = condition["condition"].as_str() {
                match cond_type {
                    "turn.isCurrent" => {
                        let player = condition["player"].as_str()
                            .unwrap_or("")
                            .replace("{actor}", caller);
                        if state["turn"].as_str() != Some(&player) {
                            return json!([]);
                        }
                    }
                    _ => {
                        // Other condition types can be added here
                    }
                }
            }
        }
    }

    let mut patches = vec![];
    
    // Apply the action - check new terminology first, fall back to builtin for compatibility
    let implementation = spec["uses"].as_str()
        .or_else(|| spec["implementation"].as_str())
        .or_else(|| spec["builtin"].as_str());
    
    println!("[DEBUG apply_action] Action {} has implementation: {:?}", action_id, implementation);
    println!("[DEBUG apply_action] About to match implementation for {}", action_id);
    
    let action_patches = match implementation {
        Some("entity.move") | Some("grid.move") | Some("moveEntity") => {
            println!("[DEBUG apply_action] Matched entity move action - calling apply_move_entity");
            apply_move_entity(bundle, state, spec, action, caller)
        }
        Some("turn.advance") | Some("nextTurn") => {
            apply_next_turn(state, caller)
        }
        Some("deck.draw") | Some("drawCard") => {
            apply_draw_card(bundle, state, spec, action, caller)
        }
        Some("deck.transfer") | Some("transferCards") => {
            apply_transfer_cards(bundle, state, spec, action, caller)
        }
        Some("grid.select") | Some("selectEntity") => {
            apply_select_entity(bundle, state, spec, action, caller)
        }
        Some("grid.moveSelected") | Some("moveSelectedEntity") => {
            apply_move_selected_entity(bundle, state, spec, action, caller)
        }
        Some("zone.reset") => {
            apply_zone_reset(bundle, state, spec, action, caller)
        }
        Some("turn.reset") => {
            apply_turn_reset(state)
        }
        Some("game.end") => {
            apply_game_end(state, spec, action, caller)
        }
        Some("phase.set") | Some("setPhase") => {
            apply_set_phase(state, spec, caller)
        }
        Some("updateScore") => {
            apply_update_score(state, spec, caller)
        }
        Some("checkScore") => {
            apply_check_score(state, spec, action, caller)
        }
        Some("countCards") => {
            apply_count_cards(state, spec, caller)
        }
        _ => {
            // Check for old-style conditions or hooks
            if spec.get("hook").is_some() {
                apply_hook(bundle, state, spec, action, caller)
            } else if spec.get("conditions").is_some() || spec.get("check").is_some() || spec.get("checks").is_some() {
                apply_conditions(bundle, state, spec, action, caller)
            } else {
                json!([])
            }
        }
    };
    
    if let Some(arr) = action_patches.as_array() {
        patches.extend_from_slice(arr);
    }
    
    // Apply patches to state before triggers
    for patch in &patches {
        apply_patch_to_state(state, patch);
    }
    
    // Process triggers - support both 'triggers' and 'then'
    let triggers = spec.get("then").or_else(|| spec.get("triggers"));
    println!("[DEBUG] Processing triggers/then for action {}: {:?}", spec["id"].as_str().unwrap_or("unknown"), triggers);
    if let Some(triggers) = triggers.and_then(|t| t.as_array()) {
        println!("[DEBUG] Found {} triggers to process", triggers.len());
        for trigger in triggers {
            let trigger_action_id = if let Some(id) = trigger.as_str() {
                // Old format: just a string
                id
            } else if let Some(act) = trigger["action"].as_str() {
                // New format: { action: "action_id" }
                act
            } else {
                continue;
            };
            
            println!("[DEBUG] Processing trigger: {}", trigger_action_id);
            // Create a trigger action
            let trigger_action = json!({ "action": trigger_action_id });
            let trigger_patches = apply_action(bundle, state, caller, &trigger_action);
            println!("[DEBUG] Trigger {} produced patches: {:?}", trigger_action_id, trigger_patches);
            if let Some(arr) = trigger_patches.as_array() {
                patches.extend_from_slice(arr);
                // Apply trigger patches to state as well
                for patch in arr {
                    apply_patch_to_state(state, patch);
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
    bundle: &Bundle,
    state: &mut Value,
    spec: &Value,
    action: &Value,
    actor: &str,
) -> Value {
    // Support both old 'params' and new 'with' terminology
    let params = spec.get("with").or_else(|| spec.get("params"));
    let params = params.unwrap_or(&spec["params"]); // fallback to params if neither exists
    
    let source_template = params["source"].as_str()
        .or_else(|| params["from"].as_str())
        .unwrap_or("");
    let source_id = source_template.replace("{actor}", actor);
    
    let target = params.get("target").or_else(|| params.get("to"));
    let target = target.unwrap_or(&params["target"]);
    let target_zone = target["zone"].as_str()
        .or_else(|| target.as_str()) // Allow direct zone reference
        .unwrap_or("");
    let target_id = target_zone.replace("{actor}", actor);

    // Extract entity from source
    let mut entity = String::new();
    let mut remove_source = false;
    
    println!("[DEBUG apply_move_entity] Moving from {} to {}", source_id, target_id);
    
    if let Some(z) = state["zones"].get_mut(&source_id) {
        println!("[DEBUG apply_move_entity] Source zone structure: {:?}", z);
        
        if z.get("infinite").is_some() {
            entity = z["infinite"].as_str().unwrap().to_string();
            // Replace {player} or {actor} with actual actor ID
            entity = entity.replace("{player}", actor).replace("{actor}", actor);
        } else if let Some(items) = z.get_mut("items").and_then(|v| v.as_array_mut()) {
            println!("[DEBUG apply_move_entity] Source has {} items", items.len());
            
            // Check if a specific card index was provided
            let mut card_index = action.get("args")
                .and_then(|args| args.get("card"))
                .and_then(|c| c.as_u64())
                .map(|idx| idx as usize);
                
            // For deck zones without a specific index, take from the top (last item)
            if card_index.is_none() && (source_id == "drawPile" || source_id == "discardPile") && !items.is_empty() {
                card_index = Some(items.len() - 1);
                println!("[DEBUG apply_move_entity] Deck zone - taking top card at index {}", card_index.unwrap());
            }
            
            let final_index = card_index.unwrap_or(0);
            println!("[DEBUG apply_move_entity] Final card index: {}", final_index);
            
            if final_index < items.len() {
                if let Some(val) = items.get(final_index) {
                    entity = val.as_str().unwrap_or("").to_string();
                    items.remove(final_index);
                    remove_source = true;
                }
            }
        } else {
            println!("[DEBUG apply_move_entity] Source zone has no items array!");
        }
    } else {
        println!("[DEBUG apply_move_entity] Source zone {} not found!", source_id);
    }
    
    if entity.is_empty() {
        println!("[DEBUG apply_move_entity] No entity found to move!");
        return json!([]);
    }
    

    // Apply to target
    let mut ops = Vec::new();
    let mut final_row = 0;
    let mut final_col = 0;
    
    if let Some(zone_val) = state["zones"].get_mut(&target_id) {
        if zone_val.is_array() {
            // grid
            let mut row = action["args"]["row"].as_u64().unwrap_or(0) as usize;
            let col = action["args"]["col"].as_u64().unwrap_or(0) as usize;
            final_row = row;
            final_col = col;
            
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
                                final_row = r;
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
        // Use the card index from args if available
        let mut card_index = action.get("args")
            .and_then(|args| args.get("card"))
            .and_then(|c| c.as_u64())
            .map(|idx| idx as usize);
            
        // For deck zones without a specific index, we removed from the top (last item)
        if card_index.is_none() && (source_id == "drawPile" || source_id == "discardPile") {
            // Get the current length after removal (it's already been removed above)
            if let Some(z) = state["zones"].get(&source_id) {
                if let Some(items) = z.get("items").and_then(|i| i.as_array()) {
                    card_index = Some(items.len()); // This was the last index before removal
                }
            }
        }
        
        let final_index = card_index.unwrap_or(0);
            
        ops.insert(0, json!({
            "op": "remove",
            "path": format!("/zones/{}/items/{}", source_id, final_index)
        }));
    }
    
    // Apply any effects specified in the verb
    if let Some(effects) = spec["params"]["effects"].as_array() {
        for effect in effects {
            match effect["type"].as_str() {
                Some("flip") => {
                    let flip_ops = apply_flip_effect(state, effect, &target_id, final_row, final_col, &entity);
                    if let Some(arr) = flip_ops.as_array() {
                        ops.extend_from_slice(arr);
                    }
                }
                Some("capture") => {
                    let capture_ops = apply_capture_effect(state, effect, &target_id, final_row, final_col, actor);
                    if let Some(arr) = capture_ops.as_array() {
                        ops.extend_from_slice(arr);
                    }
                }
                Some("transform") => {
                    let transform_ops = apply_transform_effect(bundle, state, effect, &target_id, final_row, final_col, &entity, actor);
                    if let Some(arr) = transform_ops.as_array() {
                        ops.extend_from_slice(arr);
                    }
                }
                _ => {}
            }
        }
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

fn apply_set_phase(state: &mut Value, spec: &Value, _caller: &str) -> Value {
    // Support both 'with' (new) and 'params' (old) syntax
    let params = spec.get("with").or_else(|| spec.get("params")).unwrap_or(&spec["params"]);
    let new_phase = params["phase"].as_str().unwrap_or("play");
    
    // Update phase in meta
    if let Some(meta) = state.get_mut("meta").and_then(|m| m.as_object_mut()) {
        meta.insert("currentPhase".to_string(), json!(new_phase));
    }
    
    json!([{ "op": "replace", "path": "/meta/currentPhase", "value": new_phase }])
}

fn apply_update_score(state: &mut Value, spec: &Value, caller: &str) -> Value {
    let params = &spec["params"];
    let player = params["player"].as_str()
        .unwrap_or("{actor}")
        .replace("{actor}", caller);
    
    let amount = params["amount"].as_i64().unwrap_or(0);
    let operation = params["operation"].as_str().unwrap_or("add");
    
    // Initialize scores if not present
    if state["scores"].is_null() {
        state["scores"] = json!({});
    }
    
    let current_score = state["scores"][&player].as_i64().unwrap_or(0);
    let new_score = match operation {
        "add" => current_score + amount,
        "subtract" => current_score - amount,
        "set" => amount,
        _ => current_score
    };
    
    state["scores"][&player] = json!(new_score);
    
    json!([{
        "op": "replace",
        "path": format!("/scores/{}", player),
        "value": new_score
    }])
}

fn apply_check_score(state: &mut Value, spec: &Value, _action: &Value, caller: &str) -> Value {
    let params = &spec["params"];
    let player = params["player"].as_str()
        .unwrap_or("{actor}")
        .replace("{actor}", caller);
    
    let threshold = params["threshold"].as_i64().unwrap_or(100);
    let comparison = params["comparison"].as_str().unwrap_or("gte");
    
    let score = state["scores"][&player].as_i64().unwrap_or(0);
    
    let condition_met = match comparison {
        "gte" => score >= threshold,
        "gt" => score > threshold,
        "lte" => score <= threshold,
        "lt" => score < threshold,
        "eq" => score == threshold,
        _ => false
    };
    
    if condition_met {
        if let Some(result) = params.get("result") {
            if result["gameWin"].as_str() == Some("player") {
                return json!([{
                    "op": "add",
                    "path": "/meta/gameStatus",
                    "value": {
                        "state": "ended",
                        "winner": player
                    }
                }]);
            } else if result["gameLose"].as_str() == Some("player") {
                // Find winner (other players)
                if let Some(players) = state["players"].as_array() {
                    for p in players {
                        if let Some(pid) = p["id"].as_str() {
                            if pid != player {
                                return json!([{
                                    "op": "add",
                                    "path": "/meta/gameStatus",
                                    "value": {
                                        "state": "ended",
                                        "winner": pid
                                    }
                                }]);
                            }
                        }
                    }
                }
            }
        }
    }
    
    json!([])
}

/* --------------------------------------------------------------------------
   Constraint validation for card games
   ----------------------------------------------------------------------- */
fn validate_card_constraint(
    state: &Value,
    constraint: &Value,
    source_card: &str,
    target_zone: &str,
    _target_card: Option<&str>,
) -> bool {
    // Get card properties from entities
    let source_props = get_card_properties(state, source_card);
    
    if let Some(constraint_type) = constraint.as_str() {
        match constraint_type {
            "matchingCard" => {
                // For Crazy Eights - match rank or suit, 8s are wild
                if let Some(discard_pile) = state["zones"][target_zone].get("items") {
                    if let Some(items) = discard_pile.as_array() {
                        if let Some(top_card) = items.last() {
                            let top_props = get_card_properties(state, top_card.as_str().unwrap_or(""));
                            
                            // Check if source is an 8 (wild)
                            if source_props.get("rank").and_then(|r| r.as_str()) == Some("8") {
                                return true;
                            }
                            
                            // Check rank or suit match
                            let rank_match = source_props.get("rank") == top_props.get("rank");
                            let suit_match = source_props.get("suit") == top_props.get("suit");
                            
                            return rank_match || suit_match;
                        }
                    }
                }
            }
            _ => {}
        }
    } else if let Some(constraint_obj) = constraint.as_object() {
        // Handle complex constraints
        if let Some(match_by) = constraint_obj.get("matchBy").and_then(|m| m.as_array()) {
            // For games that specify what to match
            for match_type in match_by {
                if let Some(match_str) = match_type.as_str() {
                    match match_str {
                        "rank" => {
                            // Check rank matching logic
                        }
                        "suit" => {
                            // Check suit matching logic
                        }
                        "color" => {
                            // Check color matching (red/black)
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    true // Default to allowing the move
}

fn get_card_properties(state: &Value, card_id: &str) -> serde_json::Map<String, Value> {
    // Look up card in entities
    if let Some(entities) = state.get("meta").and_then(|m| m.get("entities")) {
        if let Some(entities_array) = entities.as_array() {
            for entity in entities_array {
                if entity["id"].as_str() == Some(card_id) {
                    if let Some(props) = entity.get("props").and_then(|p| p.as_object()) {
                        return props.clone();
                    }
                }
            }
        }
    }
    serde_json::Map::new()
}

fn apply_count_cards(state: &mut Value, spec: &Value, caller: &str) -> Value {
    let params = &spec["params"];
    let zone_template = params["zone"].as_str().unwrap_or("");
    let zone = zone_template.replace("{actor}", caller);
    
    let count = if let Some(zone_data) = state["zones"].get(&zone) {
        if let Some(items) = zone_data.get("items").and_then(|i| i.as_array()) {
            items.len() as i64
        } else if zone_data.is_array() {
            // Count non-null cells in grid
            let mut count = 0;
            if let Some(rows) = zone_data.as_array() {
                for row in rows {
                    if let Some(cells) = row.as_array() {
                        count += cells.iter().filter(|c| !c.is_null()).count();
                    }
                }
            }
            count as i64
        } else {
            0
        }
    } else {
        0
    };
    
    // Store count if variable name provided
    if let Some(var_name) = params.get("storeAs").and_then(|v| v.as_str()) {
        let path = format!("/temp/{}", var_name);
        return json!([{
            "op": "add",
            "path": path,
            "value": count
        }]);
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
    println!("[DEBUG] apply_conditions called for action: {:?}, caller: {}", spec["id"], caller);
    
    // Support both 'checks' and 'conditions' for compatibility
    let conditions = spec.get("checks")
        .or_else(|| spec.get("conditions"))
        .and_then(|c| c.as_array());
    
    let conditions = match conditions {
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
            // Support both 'type' and 'builtin' fields
            let condition_type = condition["type"].as_str()
                .or_else(|| condition["builtin"].as_str());
            let condition_type = match condition_type {
                Some(t) => t,
                None => continue,
            };
            
            let condition_met = match condition_type {
                "grid.consecutiveMarks" | "consecutiveMarksInRow" => check_consecutive_marks(state, condition, check_player),
                "grid.allFilled" | "allCellsFilled" => check_all_cells_filled(state, condition),
                "deck.empty" | "deckEmpty" => check_deck_empty(state, condition),
                "zoneEmpty" => check_zone_empty(state, condition, check_player),
                "pieces.none" | "noPiecesRemaining" => check_no_pieces_remaining(state, condition, check_player),
                "moves.none" | "noValidMoves" => check_no_valid_moves(state, condition, check_player),
                "any" => {
                    // Handle nested conditions for "any" type
                    // For "any", we need to check each sub-condition and apply its result if true
                    if let Some(sub_conditions) = condition["conditions"].as_array() {
                        for sub_cond in sub_conditions {
                            let sub_met = match sub_cond["type"].as_str() {
                                Some("grid.consecutiveMarks") | Some("consecutiveMarksInRow") => check_consecutive_marks(state, sub_cond, check_player),
                                Some("grid.allFilled") | Some("allCellsFilled") => check_all_cells_filled(state, sub_cond),
                                Some("pieces.none") | Some("noPiecesRemaining") => check_no_pieces_remaining(state, sub_cond, check_player),
                                Some("moves.none") | Some("noValidMoves") => check_no_valid_moves(state, sub_cond, check_player),
                                _ => false,
                            };
                            
                            if sub_met {
                                // Apply the result from the matching sub-condition
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
                                    "gameLose" => {
                                        let winner = if check_player == "p1" { "p2" } else { "p1" };
                                        patches.push(json!({
                                            "op": "add",
                                            "path": "/meta/gameStatus",
                                            "value": {
                                                "state": "ended",
                                                "winner": winner,
                                                "tie": false
                                            }
                                        }));
                                        return json!(patches);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        false // No sub-condition matched
                    } else {
                        false
                    }
                },
                _ => false,
            };
            
            if condition_met {
                // Apply the result - support both string and array formats
                if let Some(result_array) = condition.get("result").and_then(|r| r.as_array()) {
                    // Handle array format like [{ "gameWin": "actor" }]
                    for result_item in result_array {
                        if let Some(game_win) = result_item.get("gameWin").and_then(|v| v.as_str()) {
                            let winner = if game_win == "actor" { check_player } else { game_win };
                            patches.push(json!({
                                "op": "add",
                                "path": "/meta/gameStatus",
                                "value": {
                                    "state": "ended",
                                    "winner": winner,
                                    "tie": false
                                }
                            }));
                            return json!(patches);
                        } else if result_item.get("gameTie").is_some() {
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
                    }
                } else if let Some(result) = condition["result"].as_str() {
                    // Handle old string format
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
                        "gameLose" => {
                            // The current player loses, so the other player wins
                            let winner = if check_player == "p1" { "p2" } else { "p1" };
                            patches.push(json!({
                                "op": "add",
                                "path": "/meta/gameStatus",
                                "value": {
                                    "state": "ended",
                                    "winner": winner,
                                    "tie": false
                                }
                            }));
                            return json!(patches);
                        }
                        _ => {}
                    }
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

fn check_deck_empty(state: &Value, condition: &Value) -> bool {
    let zone_id = condition["params"]["zone"].as_str().unwrap_or("");
    if let Some(zone) = state["zones"].get(zone_id) {
        if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
            return items.is_empty();
        }
    }
    false
}

fn check_zone_empty(state: &Value, condition: &Value, actor: &str) -> bool {
    let zone_template = condition["params"]["zone"].as_str().unwrap_or("");
    let zone_id = zone_template.replace("{actor}", actor);
    
    if let Some(zone) = state["zones"].get(&zone_id) {
        if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
            return items.is_empty();
        }
        // Check grid zones
        if let Some(rows) = zone.as_array() {
            for row in rows {
                if let Some(cells) = row.as_array() {
                    for cell in cells {
                        if !cell.is_null() {
                            return false;
                        }
                    }
                }
            }
            return true;
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

/* --------------------------------------------------------------------------
   Card game mechanics - draw cards from deck
   ----------------------------------------------------------------------- */
fn apply_draw_card(
    _bundle: &Bundle,
    state: &mut Value,
    spec: &Value,
    _action: &Value,
    actor: &str,
) -> Value {
    let source = spec["params"]["source"].as_str().unwrap_or("");
    let target_template = spec["params"]["target"].as_str().unwrap_or("");
    let target = target_template.replace("{actor}", actor);
    let count = spec["params"]["count"].as_u64().unwrap_or(1) as usize;
    
    let mut ops = Vec::new();
    
    // Get cards from deck
    let mut drawn_cards = Vec::new();
    if let Some(deck) = state["zones"][source].get_mut("items").and_then(|v| v.as_array_mut()) {
        for _ in 0..count {
            if let Some(card) = deck.pop() {
                drawn_cards.push(card);
            } else {
                break; // No more cards in deck
            }
        }
    }
    
    if drawn_cards.is_empty() {
        return json!([]);
    }
    
    // Add to target hand
    if let Some(hand) = state["zones"][&target].get_mut("items").and_then(|v| v.as_array_mut()) {
        for card in drawn_cards.iter() {
            hand.push(card.clone());
            ops.push(json!({
                "op": "add",
                "path": format!("/zones/{}/items/-", target),
                "value": card
            }));
        }
        
        // Remove operations were already handled by deck.pop() above
        let deck_len = state["zones"][source]["items"].as_array().map(|a| a.len()).unwrap_or(0);
        for i in 0..drawn_cards.len() {
            ops.insert(0, json!({
                "op": "remove",
                "path": format!("/zones/{}/items/{}", source, deck_len + i)
            }));
        }
    }
    
    Value::Array(ops)
}

/* --------------------------------------------------------------------------
   Transfer cards between players (for Go Fish asking)
   ----------------------------------------------------------------------- */
fn apply_transfer_cards(
    bundle: &Bundle,
    state: &mut Value,
    _spec: &Value,
    action: &Value,
    actor: &str,
) -> Value {
    let target_player = action["args"]["targetPlayer"].as_str().unwrap_or("");
    let requested_rank = action["args"]["rank"].as_str().unwrap_or("");
    
    let source_zone = format!("hand_{}", target_player);
    let target_zone = format!("hand_{}", actor);
    
    let mut ops = Vec::new();
    let mut transferred_cards = Vec::new();
    
    // Get entities from bundle
    let entities = bundle.entities.as_array();
    
    // Find and remove matching cards from target player's hand
    if let Some(source_hand) = state["zones"][&source_zone].get_mut("items").and_then(|v| v.as_array_mut()) {
        let mut i = 0;
        while i < source_hand.len() {
            let card = &source_hand[i];
            
            // Check if card matches requested rank
            if let Some(entities_array) = entities {
                if let Some(entity) = entities_array.iter().find(|e| &e["id"] == card) {
                    if entity["props"]["rank"].as_str() == Some(requested_rank) {
                        transferred_cards.push(source_hand.remove(i));
                        continue;
                    }
                }
            }
            i += 1;
        }
    }
    
    // Add cards to requester's hand
    if !transferred_cards.is_empty() {
        if let Some(target_hand) = state["zones"][&target_zone].get_mut("items").and_then(|v| v.as_array_mut()) {
            for card in &transferred_cards {
                target_hand.push(card.clone());
                ops.push(json!({
                    "op": "add",
                    "path": format!("/zones/{}/items/-", target_zone),
                    "value": card
                }));
            }
            
            // Remove from source (in reverse order to maintain indices)
            if let Some(source_hand) = state["zones"][&source_zone].as_array() {
                let original_len = source_hand.len() + transferred_cards.len();
                for i in 0..transferred_cards.len() {
                    ops.push(json!({
                        "op": "remove",
                        "path": format!("/zones/{}/items/{}", source_zone, original_len - i - 1)
                    }));
                }
            }
        }
    }
    
    // Add metadata about the transfer result
    ops.push(json!({
        "op": "add",
        "path": "/meta/lastAskResult",
        "value": {
            "asker": actor,
            "target": target_player,
            "rank": requested_rank,
            "cardsTransferred": transferred_cards.len()
        }
    }));
    
    Value::Array(ops)
}

fn get_flips_in_direction(
    board: &Value,
    start_row: usize,
    start_col: usize,
    dr: i32,
    dc: i32,
    player_piece: &str,
) -> Vec<(usize, usize)> {
    let flips = Vec::new();
    let mut temp_flips = Vec::new();
    
    let board_array = match board.as_array() {
        Some(arr) => arr,
        None => return flips,
    };
    
    let board_size = board_array.len();
    let mut r = start_row as i32 + dr;
    let mut c = start_col as i32 + dc;
    
    // Get the opponent's piece type
    let opponent_piece = if player_piece.contains("_p1") {
        player_piece.replace("_p1", "_p2")
    } else {
        player_piece.replace("_p2", "_p1")
    };
    
    // Look for opponent pieces
    while r >= 0 && r < board_size as i32 && c >= 0 && c < board_size as i32 {
        let row_idx = r as usize;
        let col_idx = c as usize;
        
        if let Some(row_array) = board_array[row_idx].as_array() {
            if col_idx < row_array.len() {
                match row_array[col_idx].as_str() {
                    Some(piece) if piece == opponent_piece => {
                        temp_flips.push((row_idx, col_idx));
                    }
                    Some(piece) if piece == player_piece => {
                        // Found our own piece - all temp_flips are valid
                        return temp_flips;
                    }
                    _ => {
                        // Empty cell or edge - no flips in this direction
                        return flips;
                    }
                }
            }
        }
        
        r += dr;
        c += dc;
    }
    
    // Hit edge without finding our piece - no flips
    flips
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

/* --------------------------------------------------------------------------
   Effect Functions for moveEntity
   ----------------------------------------------------------------------- */

fn apply_flip_effect(
    state: &mut Value,
    effect: &Value,
    zone_id: &str,
    row: usize,
    col: usize,
    entity: &str,
) -> Value {
    let mut ops = Vec::new();
    let pattern = effect["pattern"].as_str().unwrap_or("straight");
    
    if pattern == "straight" {
        // Similar to Reversi - flip pieces in straight lines
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        
        for (dr, dc) in directions.iter() {
            let flips = get_flips_in_direction(&state["zones"][zone_id], row, col, *dr, *dc, entity);
            for (flip_row, flip_col) in flips {
                ops.push(json!({
                    "op": "replace",
                    "path": format!("/zones/{}/{}/{}", zone_id, flip_row, flip_col),
                    "value": entity
                }));
            }
        }
    }
    
    json!(ops)
}

fn apply_capture_effect(
    state: &mut Value,
    effect: &Value,
    zone_id: &str,
    row: usize,
    col: usize,
    actor: &str,
) -> Value {
    let mut ops = Vec::new();
    let capture_type = effect["captureType"].as_str().unwrap_or("adjacent");
    
    if capture_type == "adjacent" {
        // Capture all adjacent opponent pieces
        let directions = [
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1)
        ];
        
        if let Some(zone) = state["zones"][zone_id].as_array() {
            for (dr, dc) in directions.iter() {
                let adj_row = row as i32 + dr;
                let adj_col = col as i32 + dc;
                
                if adj_row >= 0 && adj_row < zone.len() as i32 && adj_col >= 0 {
                    if let Some(cell) = zone.get(adj_row as usize)
                        .and_then(|r| r.as_array())
                        .and_then(|r| r.get(adj_col as usize))
                        .and_then(|c| c.as_str()) {
                        
                        // Check if it's an opponent's piece
                        if !cell.is_empty() && !cell.contains(&format!("_{}", actor)) {
                            ops.push(json!({
                                "op": "replace",
                                "path": format!("/zones/{}/{}/{}", zone_id, adj_row, adj_col),
                                "value": null
                            }));
                        }
                    }
                }
            }
        }
    }
    
    json!(ops)
}

fn apply_transform_effect(
    _bundle: &Bundle,
    _state: &mut Value,
    effect: &Value,
    zone_id: &str,
    row: usize,
    col: usize,
    entity: &str,
    actor: &str,
) -> Value {
    let mut ops = Vec::new();
    
    // Check if transformation condition is met
    if let Some(condition) = effect.get("when") {
        let condition_type = condition["type"].as_str().unwrap_or("");
        
        if condition_type == "reachesRow" {
            let target_row = condition["row"].as_str().unwrap_or("");
            let should_transform = match target_row {
                "opposite" => {
                    (entity.contains("_p1") && row == 0) ||
                    (entity.contains("_p2") && row == 7)
                }
                _ => {
                    if let Ok(specific_row) = target_row.parse::<usize>() {
                        row == specific_row
                    } else {
                        false
                    }
                }
            };
            
            if should_transform {
                if let Some(to_template) = effect["to"].as_str() {
                    let new_entity = to_template.replace("{actor}", actor);
                    ops.push(json!({
                        "op": "replace",
                        "path": format!("/zones/{}/{}/{}", zone_id, row, col),
                        "value": new_entity
                    }));
                }
            }
        }
    }
    
    json!(ops)
}

/* --------------------------------------------------------------------------
   Check if player has no pieces remaining (for checkers win condition)
   ----------------------------------------------------------------------- */
fn check_no_pieces_remaining(state: &Value, condition: &Value, player: &str) -> bool {
    let zone_name = condition["params"]["zone"].as_str().unwrap_or("");
    
    // For checkers, check if player has any pieces OR kings
    if let Some(board) = state["zones"][zone_name].as_array() {
        for row in board {
            if let Some(cells) = row.as_array() {
                for cell in cells {
                    if let Some(cell_entity) = cell.as_str() {
                        // Check if this entity belongs to the player
                        if cell_entity.ends_with(&format!("_{}", player)) {
                            return false; // Found at least one piece belonging to this player
                        }
                    }
                }
            }
        }
    }
    
    true // No pieces found for this player
}

/* --------------------------------------------------------------------------
   Check if player has no valid moves (for checkers stalemate)
   ----------------------------------------------------------------------- */
fn check_no_valid_moves(_state: &Value, _condition: &Value, _player: &str) -> bool {
    // This would require checking all pieces and their possible moves
    // For now, return false to not trigger this condition
    // In a full implementation, this would check if the player has any legal moves
    false
}

/* --------------------------------------------------------------------------
   Select entity for two-step interactions (e.g., checkers piece selection)
   ----------------------------------------------------------------------- */
fn apply_select_entity(
    _bundle: &Bundle,
    state: &mut Value,
    spec: &Value,
    action: &Value,
    actor: &str,
) -> Value {
    let zone = spec["params"]["zone"].as_str().unwrap_or("");
    let row = action["args"]["row"].as_u64().unwrap_or(0) as usize;
    let col = action["args"]["col"].as_u64().unwrap_or(0) as usize;
    
    // Store the selection in state
    let selection = json!({
        "actor": actor,
        "zone": zone,
        "row": row,
        "col": col
    });
    
    state["meta"]["selection"] = selection.clone();
    
    json!([{
        "op": "add",
        "path": "/meta/selection",
        "value": selection
    }])
}

/* --------------------------------------------------------------------------
   Move the previously selected entity (for checkers)
   ----------------------------------------------------------------------- */
fn apply_move_selected_entity(
    bundle: &Bundle,
    state: &mut Value,
    spec: &Value,
    action: &Value,
    actor: &str,
) -> Value {
    // Get the current selection and extract values we need
    let (source_zone, source_row, source_col) = {
        let selection = match state.get("meta").and_then(|m| m.get("selection")) {
            Some(sel) => sel,
            None => return json!([]),
        };
        
        // Verify the selection belongs to the current actor
        if selection["actor"].as_str() != Some(actor) {
            return json!([]);
        }
        
        (
            selection["zone"].as_str().unwrap_or("").to_string(),
            selection["row"].as_u64().unwrap_or(0) as usize,
            selection["col"].as_u64().unwrap_or(0) as usize,
        )
    };
    
    let target_zone = spec["params"]["target"]["zone"].as_str().unwrap_or("");
    let target_row = action["args"]["row"].as_u64().unwrap_or(0) as usize;
    let target_col = action["args"]["col"].as_u64().unwrap_or(0) as usize;
    
    let mut ops = Vec::new();
    
    // Get the entity from source position
    let entity = if let Some(zone_val) = state["zones"].get(&source_zone) {
        if let Some(row_arr) = zone_val.as_array().and_then(|z| z.get(source_row)) {
            if let Some(cell) = row_arr.as_array().and_then(|r| r.get(source_col)) {
                cell.as_str().unwrap_or("").to_string()
            } else {
                return json!([]);
            }
        } else {
            return json!([]);
        }
    } else {
        return json!([]);
    };
    
    if entity.is_empty() {
        return json!([]);
    }
    
    // Check if this is a capture move (diagonal with a piece to capture)
    let is_capture = (source_row as i32 - target_row as i32).abs() == 2 &&
                     (source_col as i32 - target_col as i32).abs() == 2;
    
    if is_capture {
        // Calculate the captured piece position
        let captured_row = ((source_row + target_row) / 2) as usize;
        let captured_col = ((source_col + target_col) / 2) as usize;
        
        // Remove the captured piece
        if let Some(zone_val) = state["zones"].get_mut(&target_zone) {
            if let Some(row_arr) = zone_val.as_array_mut().and_then(|z| z.get_mut(captured_row)) {
                if let Some(cells) = row_arr.as_array_mut() {
                    cells[captured_col] = Value::Null;
                    ops.push(json!({
                        "op": "replace",
                        "path": format!("/zones/{}/{}/{}", target_zone, captured_row, captured_col),
                        "value": null
                    }));
                }
            }
        }
    }
    
    // We'll handle promotion through the transform effect system now
    let final_entity = entity.clone();
    
    // Move the piece to target
    if let Some(zone_val) = state["zones"].get_mut(&target_zone) {
        if let Some(row_arr) = zone_val.as_array_mut().and_then(|z| z.get_mut(target_row)) {
            if let Some(cells) = row_arr.as_array_mut() {
                if cells[target_col].is_null() {
                    cells[target_col] = Value::String(final_entity.clone());
                    ops.push(json!({
                        "op": "replace",
                        "path": format!("/zones/{}/{}/{}", target_zone, target_row, target_col),
                        "value": final_entity.clone()
                    }));
                }
            }
        }
    }
    
    // Remove from source
    if let Some(zone_val) = state["zones"].get_mut(&source_zone) {
        if let Some(row_arr) = zone_val.as_array_mut().and_then(|z| z.get_mut(source_row)) {
            if let Some(cells) = row_arr.as_array_mut() {
                cells[source_col] = Value::Null;
                ops.push(json!({
                    "op": "replace",
                    "path": format!("/zones/{}/{}/{}", source_zone, source_row, source_col),
                    "value": null
                }));
            }
        }
    }
    
    // Clear the selection
    ops.push(json!({
        "op": "remove",
        "path": "/meta/selection"
    }));
    
    // Apply any transform effects (like king promotion)
    if let Some(effects) = spec["params"]["effects"].as_array() {
        for effect in effects {
            if effect["type"].as_str() == Some("transform") {
                let transform_ops = apply_transform_effect(bundle, state, effect, &target_zone, target_row, target_col, &final_entity, actor);
                if let Some(arr) = transform_ops.as_array() {
                    ops.extend_from_slice(arr);
                }
            }
        }
    }
    
    // Check if this was a capture move by a king and more captures are available
    if is_capture && final_entity.contains("king_") {
        // Check if there are more captures available from the new position
        let mut has_more_captures = false;
        
        // Check all four diagonal directions for potential captures
        for dr in [-2i32, 2].iter() {
            for dc in [-2i32, 2].iter() {
                let next_row = target_row as i32 + dr;
                let next_col = target_col as i32 + dc;
                
                if next_row >= 0 && next_row < 8 && next_col >= 0 && next_col < 8 {
                    let mid_row = ((target_row as i32 + next_row) / 2) as usize;
                    let mid_col = ((target_col as i32 + next_col) / 2) as usize;
                    
                    // Check if the destination is empty and there's an opponent piece to jump
                    if let Some(zone_val) = state["zones"].get(&target_zone) {
                        if let Some(dest_cell) = zone_val.as_array()
                            .and_then(|z| z.get(next_row as usize))
                            .and_then(|r| r.as_array())
                            .and_then(|r| r.get(next_col as usize)) {
                            
                            if dest_cell.is_null() {
                                if let Some(mid_cell) = zone_val.as_array()
                                    .and_then(|z| z.get(mid_row))
                                    .and_then(|r| r.as_array())
                                    .and_then(|r| r.get(mid_col))
                                    .and_then(|c| c.as_str()) {
                                    
                                    // Check if it's an opponent's piece
                                    if !mid_cell.contains(&format!("_{}", actor)) && !mid_cell.is_empty() {
                                        has_more_captures = true;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if has_more_captures {
                break;
            }
        }
        
        // If more captures are available, automatically select the king at its new position
        if has_more_captures {
            ops.push(json!({
                "op": "add",
                "path": "/meta/selection",
                "value": {
                    "actor": actor,
                    "zone": target_zone,
                    "row": target_row,
                    "col": target_col
                }
            }));
            
            // Don't advance turn - same player continues
            return Value::Array(ops);
        }
    }
    
    // Advance turn if we're not continuing with multiple jumps
    let next_turn = {
        if let Some(players) = state["players"].as_array() {
            players
                .iter()
                .position(|p| p["id"].as_str() == Some(actor))
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
        ops.push(json!({ "op": "replace", "path": "/turn", "value": next }));
    }
    
    Value::Array(ops)
}

fn apply_zone_reset(
    bundle: &Bundle,
    _state: &mut Value,
    spec: &Value,
    _action: &Value,
    actor: &str,
) -> Value {
    let mut ops = Vec::new();
    
    // Support both old 'params' and new 'with' terminology
    let params = spec.get("with").or_else(|| spec.get("params"));
    let params = params.unwrap_or(&spec["params"]);
    
    if let Some(zones) = params["zones"].as_array() {
        for zone_ref in zones {
            let zone_id = zone_ref.as_str().unwrap_or("").replace("{actor}", actor);
            
            // Find the zone definition in the bundle
            if let Some(zone_def) = bundle.zones.as_object()
                .and_then(|zones| zones.get(&zone_id)) {
                
                // Reset based on zone type
                if let Some(zone_type) = zone_def["type"].as_str() {
                    match zone_type {
                        "grid" => {
                            // Reset grid to empty cells
                            if let Some(rows) = zone_def["rows"].as_u64() {
                                if let Some(cols) = zone_def["cols"].as_u64() {
                                    let empty_grid: Vec<Vec<Value>> = (0..rows)
                                        .map(|_| vec![Value::Null; cols as usize])
                                        .collect();
                                    ops.push(json!({
                                        "op": "replace",
                                        "path": format!("/zones/{}", zone_id),
                                        "value": empty_grid
                                    }));
                                }
                            }
                        }
                        "list" => {
                            // Reset list to empty or infinite source
                            if zone_def.get("infinite").is_some() {
                                // Keep infinite zones as is
                            } else {
                                ops.push(json!({
                                    "op": "replace",
                                    "path": format!("/zones/{}/items", zone_id),
                                    "value": []
                                }));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    
    Value::Array(ops)
}

fn apply_turn_reset(state: &Value) -> Value {
    let mut ops = Vec::new();
    
    // Reset to first player
    if let Some(players) = state["players"].as_array() {
        if let Some(first_player) = players.first() {
            if let Some(player_id) = first_player["id"].as_str() {
                ops.push(json!({
                    "op": "replace",
                    "path": "/turn",
                    "value": player_id
                }));
            }
        }
    }
    
    // Reset game status
    ops.push(json!({
        "op": "replace",
        "path": "/meta/gameStatus",
        "value": {
            "state": "playing"
        }
    }));
    
    Value::Array(ops)
}

fn apply_game_end(
    _state: &Value,
    spec: &Value,
    _action: &Value,
    _caller: &str,
) -> Value {
    let mut ops = Vec::new();
    
    // Support both old 'params' and new 'with' terminology
    let params = spec.get("with").or_else(|| spec.get("params"));
    let params = params.unwrap_or(&spec["params"]);
    
    // Determine game result
    let game_status = if let Some(winner) = params.get("winner").and_then(|w| w.as_str()) {
        json!({
            "state": "ended",
            "winner": winner
        })
    } else if params.get("result").and_then(|r| r.as_str()) == Some("tie") {
        json!({
            "state": "ended",
            "result": "tie"
        })
    } else {
        json!({
            "state": "ended"
        })
    };
    
    ops.push(json!({
        "op": "replace",
        "path": "/meta/gameStatus",
        "value": game_status
    }));
    
    Value::Array(ops)
}
