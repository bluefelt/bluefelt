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
            let contents = zone.get("contents").unwrap_or(&Value::String("empty".to_string()));
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
        let mut rows = Vec::new();
        for row in arr {
            if let Some(cells) = row.as_sequence() {
                rows.push(cells.iter().cloned().collect());
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
pub fn apply_verb(bundle: &Bundle, state: &mut Value, action: &Value) -> Value {
    let verb_id = action["verb"].as_str().unwrap_or("");
    let verb_spec = bundle
        .verbs
        .as_array()
        .and_then(|v| v.iter().find(|x| x["id"].as_str() == Some(verb_id)));
    let Some(spec) = verb_spec else { return json!([]) };
    if spec["builtin"].as_str() == Some("moveEntity") {
        return apply_move_entity(bundle, state, spec, action);
    }
    json!([])
}

fn apply_move_entity(_bundle: &Bundle, state: &mut Value, spec: &Value, action: &Value) -> Value {
    let actor = state["turn"].as_str().unwrap_or("p1");
    let source_template = spec["params"]["source"].as_str().unwrap_or("");
    let source_id = source_template.replace("{actor}", actor);
    let target_zone = spec["params"]["target"]["zone"].as_str().unwrap_or("");
    let target_id = target_zone.replace("{actor}", actor);

    // Determine entity without modifying state
    let (entity, removable) = {
        let zones = state.get("zones").and_then(|z| z.as_object()).unwrap();
        let Some(src) = zones.get(&source_id) else { return json!([]) };
        if let Some(inf) = src.get("infinite").and_then(|v| v.as_str()) {
            (inf.to_string(), false)
        } else if let Some(items) = src.get("items").and_then(|v| v.as_array()) {
            if let Some(val) = items.first().and_then(|v| v.as_str()) {
                (val.to_string(), true)
            } else {
                return json!([]);
            }
        } else {
            return json!([]);
        }
    };

    // Validate target without mutating
    let valid_target = {
        let zones = state.get("zones").and_then(|z| z.as_object()).unwrap();
        let Some(tgt) = zones.get(&target_id) else { return json!([]) };
        if tgt.is_array() {
            let row = action["args"]["row"].as_u64().unwrap_or(0) as usize;
            let col = action["args"]["col"].as_u64().unwrap_or(0) as usize;
            if let Some(row_arr) = tgt.as_array().and_then(|r| r.get(row)) {
                if let Some(cells) = row_arr.as_array() {
                    if cells.get(col).map(|c| c.is_null()).unwrap_or(false) {
                        Some((row, col))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else if tgt.get("items").is_some() {
            Some((usize::MAX, usize::MAX))
        } else {
            None
        }
    };

    let Some(tgt_pos) = valid_target else { return json!([]) };

    // Perform state mutation now that validation succeeded
    let mut ops = Vec::new();
    if removable {
        if let Some(src_zone) = state["zones"].get_mut(&source_id) {
            if let Some(items) = src_zone.get_mut("items").and_then(|v| v.as_array_mut()) {
                if !items.is_empty() {
                    items.remove(0);
                    ops.push(json!({
                        "op": "remove",
                        "path": format!("/zones/{}/items/0", source_id)
                    }));
                }
            }
        }
    }

    if let Some(zone_val) = state["zones"].get_mut(&target_id) {
        if zone_val.is_array() {
            let row = tgt_pos.0;
            let col = tgt_pos.1;
            if let Some(row_arr) = zone_val.as_array_mut().and_then(|r| r.get_mut(row)) {
                if let Some(cells) = row_arr.as_array_mut() {
                    cells[col] = Value::String(entity.clone());
                    ops.push(json!({
                        "op": "replace",
                        "path": format!("/zones/{}/{}/{}", target_id, row, col),
                        "value": entity
                    }));
                }
            }
        } else if zone_val.get("items").is_some() {
            let items = zone_val.get_mut("items").unwrap().as_array_mut().unwrap();
            items.push(Value::String(entity.clone()));
            ops.push(json!({
                "op": "add",
                "path": format!("/zones/{}/items/-", target_id),
                "value": entity
            }));
        }
    }

    // rotate turn
    if let Some(players) = state["players"].as_array() {
        if let Some(idx) = players.iter().position(|p| p["id"].as_str() == Some(actor)) {
            let next = players[(idx + 1) % players.len()]["id"].as_str().unwrap();
            state["turn"] = Value::String(next.to_string());
            ops.push(json!({"op":"replace","path":"/turn","value":next}));
        }
    }

    Value::Array(ops)
}
