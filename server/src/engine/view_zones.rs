/// View Zone Module
/// 
/// This module implements server-side computation for view zones - special zones
/// that display strategic information rather than containing entities.
/// 
/// View zones provide computed game state, statistics, and summaries that clients
/// can render generically without game-specific logic.

use serde::{Serialize, Deserialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use crate::bundle::Bundle;
use regex::Regex;

/// View zone render data with computed information
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename = "view")]
pub struct ViewZoneRenderData {
    pub view_type: String,
    pub data: ViewZoneData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<ViewFormat>,
}

/// Structured data for view zones
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewZoneData {
    /// Per-player data fields
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub players: HashMap<String, HashMap<String, Value>>,
    
    /// Shared/global data fields
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub shared: HashMap<String, Value>,
    
    /// Metadata about the view
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ViewMetadata>,
}

/// Metadata for view zones
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<u64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_frequency: Option<String>, // "realtime", "turn", "phase"
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>, // Field labels for UI
}

/// Display format hints for view zones
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ViewFormat {
    /// Preferred display style
    pub style: ViewStyle,
    
    /// Sorting preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    
    /// Whether to show differences/changes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_delta: Option<bool>,
    
    /// Maximum entries for log-style views
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_entries: Option<usize>,
}

/// View display styles
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ViewStyle {
    Table,       // Tabular data with rows/columns
    List,        // Simple list of items
    Cards,       // Card-based layout
    Chart,       // Visual chart/graph
    Log,         // Scrolling log entries
    Summary,     // Compact summary
}

/// Compute view zone data based on zone definition
pub fn compute_view_zone_data(
    zone_id: &str,
    zone_def: &Value,
    game_state: &Value,
    player_names: &[String],
    _bundle: &Bundle,
) -> ViewZoneRenderData {
    let view_type = zone_def.get("viewType")
        .and_then(|v| v.as_str())
        .unwrap_or("strategic")
        .to_string();
    
    let empty_vec = Vec::new();
    let view_data_def = zone_def.get("viewData")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);
    
    // Compute data for each field definition
    let mut players: HashMap<String, HashMap<String, Value>> = HashMap::new();
    let mut shared: HashMap<String, Value> = HashMap::new();
    let mut labels: HashMap<String, String> = HashMap::new();
    
    for field_def in view_data_def {
        if let Some(field_obj) = field_def.as_object() {
            let field_name = field_obj.get("field")
                .and_then(|f| f.as_str())
                .unwrap_or("")
                .to_string();
            
            let label = field_obj.get("label")
                .and_then(|l| l.as_str())
                .unwrap_or(&field_name)
                .to_string();
            
            labels.insert(field_name.clone(), label);
            
            let source = field_obj.get("source")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            
            let per_player = field_obj.get("perPlayer")
                .and_then(|p| p.as_bool())
                .unwrap_or(false);
            
            if per_player {
                // Compute value for each player
                for (idx, player_name) in player_names.iter().enumerate() {
                    let player_id = format!("p{}", idx + 1);
                    let player_source = source
                        .replace("{player}", &player_id)
                        .replace("{playerName}", player_name);
                    
                    let value = evaluate_view_expression(&player_source, game_state, &player_id);
                    
                    players
                        .entry(player_id)
                        .or_insert_with(HashMap::new)
                        .insert(field_name.clone(), value);
                }
            } else {
                // Compute shared value
                let value = evaluate_view_expression(source, game_state, "");
                shared.insert(field_name, value);
            }
        }
    }
    
    // Determine format based on view type
    let format = determine_view_format(&view_type, zone_def);
    
    ViewZoneRenderData {
        view_type,
        data: ViewZoneData {
            players,
            shared,
            meta: Some(ViewMetadata {
                last_updated: None,
                update_frequency: Some("turn".to_string()),
                labels: Some(labels),
            }),
        },
        format,
    }
}

/// Evaluate a view expression to compute its value
fn evaluate_view_expression(expression: &str, game_state: &Value, player_id: &str) -> Value {
    // Handle count() expressions
    if expression.starts_with("count(") && expression.ends_with(")") {
        let zone_path = &expression[6..expression.len()-1];
        return evaluate_count_expression(zone_path, game_state);
    }
    
    // Handle sum() expressions
    if expression.starts_with("sum(") && expression.ends_with(")") {
        let path = &expression[4..expression.len()-1];
        return evaluate_sum_expression(path, game_state);
    }
    
    // Handle avg() expressions
    if expression.starts_with("avg(") && expression.ends_with(")") {
        let path = &expression[4..expression.len()-1];
        return evaluate_avg_expression(path, game_state);
    }
    
    // Handle max() expressions
    if expression.starts_with("max(") && expression.ends_with(")") {
        let path = &expression[4..expression.len()-1];
        return evaluate_max_expression(path, game_state);
    }
    
    // Handle min() expressions
    if expression.starts_with("min(") && expression.ends_with(")") {
        let path = &expression[4..expression.len()-1];
        return evaluate_min_expression(path, game_state);
    }
    
    // Handle direct state paths
    if expression.starts_with("state.") || expression.starts_with("/") {
        return evaluate_state_path(expression, game_state, player_id);
    }
    
    // Handle mathematical expressions
    if expression.contains('/') || expression.contains('*') || 
       expression.contains('+') || expression.contains('-') {
        return evaluate_math_expression(expression, game_state, player_id);
    }
    
    // Default: return as string
    json!(expression)
}

/// Evaluate count() expressions
fn evaluate_count_expression(zone_path: &str, game_state: &Value) -> Value {
    let zones = game_state.get("zones").and_then(|z| z.as_object());
    
    if let Some(zones_obj) = zones {
        if let Some(zone) = zones_obj.get(zone_path) {
            if let Some(arr) = zone.as_array() {
                return json!(arr.len());
            } else if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                return json!(items.len());
            }
        }
    }
    
    json!(0)
}

/// Evaluate sum() expressions
fn evaluate_sum_expression(path: &str, game_state: &Value) -> Value {
    // Parse the path to determine what to sum
    if path.contains(".") {
        // Sum a property across zone items (e.g., "treasury_p1.value")
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 2 {
            let zone_path = parts[0];
            let property = parts[1];
            
            if let Some(zones) = game_state.get("zones").and_then(|z| z.as_object()) {
                if let Some(zone) = zones.get(zone_path) {
                    let mut sum = 0i64;
                    
                    // Handle different zone formats
                    if let Some(items) = zone.as_array() {
                        for item in items {
                            if let Some(prop_val) = item.get(property) {
                                if let Some(num) = prop_val.as_i64() {
                                    sum += num;
                                } else if let Some(num) = prop_val.as_f64() {
                                    sum += num as i64;
                                }
                            }
                        }
                    } else if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                        for item in items {
                            if let Some(prop_val) = item.get(property) {
                                if let Some(num) = prop_val.as_i64() {
                                    sum += num;
                                } else if let Some(num) = prop_val.as_f64() {
                                    sum += num as i64;
                                }
                            }
                        }
                    }
                    
                    return json!(sum);
                }
            }
        }
    }
    
    json!(0)
}

/// Evaluate avg() expressions
fn evaluate_avg_expression(path: &str, game_state: &Value) -> Value {
    // Similar to sum but calculate average
    if path.contains(".") {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 2 {
            let zone_path = parts[0];
            let property = parts[1];
            
            if let Some(zones) = game_state.get("zones").and_then(|z| z.as_object()) {
                if let Some(zone) = zones.get(zone_path) {
                    let mut sum = 0.0;
                    let mut count = 0;
                    
                    // Handle different zone formats
                    let items = if let Some(items) = zone.as_array() {
                        items
                    } else if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                        items
                    } else {
                        return json!(0);
                    };
                    
                    for item in items {
                        if let Some(prop_val) = item.get(property) {
                            if let Some(num) = prop_val.as_f64() {
                                sum += num;
                                count += 1;
                            } else if let Some(num) = prop_val.as_i64() {
                                sum += num as f64;
                                count += 1;
                            }
                        }
                    }
                    
                    if count > 0 {
                        return json!(sum / count as f64);
                    }
                }
            }
        }
    }
    
    json!(0)
}

/// Evaluate max() expressions
fn evaluate_max_expression(path: &str, game_state: &Value) -> Value {
    if path.contains(".") {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 2 {
            let zone_path = parts[0];
            let property = parts[1];
            
            if let Some(zones) = game_state.get("zones").and_then(|z| z.as_object()) {
                if let Some(zone) = zones.get(zone_path) {
                    let mut max_val: Option<f64> = None;
                    
                    // Handle different zone formats
                    let items = if let Some(items) = zone.as_array() {
                        items
                    } else if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                        items
                    } else {
                        return json!(null);
                    };
                    
                    for item in items {
                        if let Some(prop_val) = item.get(property) {
                            let val = if let Some(num) = prop_val.as_f64() {
                                num
                            } else if let Some(num) = prop_val.as_i64() {
                                num as f64
                            } else {
                                continue;
                            };
                            
                            max_val = Some(max_val.map(|m| m.max(val)).unwrap_or(val));
                        }
                    }
                    
                    if let Some(max) = max_val {
                        return json!(max);
                    }
                }
            }
        }
    }
    
    json!(null)
}

/// Evaluate min() expressions
fn evaluate_min_expression(path: &str, game_state: &Value) -> Value {
    if path.contains(".") {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() == 2 {
            let zone_path = parts[0];
            let property = parts[1];
            
            if let Some(zones) = game_state.get("zones").and_then(|z| z.as_object()) {
                if let Some(zone) = zones.get(zone_path) {
                    let mut min_val: Option<f64> = None;
                    
                    // Handle different zone formats
                    let items = if let Some(items) = zone.as_array() {
                        items
                    } else if let Some(items) = zone.get("items").and_then(|i| i.as_array()) {
                        items
                    } else {
                        return json!(null);
                    };
                    
                    for item in items {
                        if let Some(prop_val) = item.get(property) {
                            let val = if let Some(num) = prop_val.as_f64() {
                                num
                            } else if let Some(num) = prop_val.as_i64() {
                                num as f64
                            } else {
                                continue;
                            };
                            
                            min_val = Some(min_val.map(|m| m.min(val)).unwrap_or(val));
                        }
                    }
                    
                    if let Some(min) = min_val {
                        return json!(min);
                    }
                }
            }
        }
    }
    
    json!(null)
}

/// Evaluate direct state path references
fn evaluate_state_path(path: &str, game_state: &Value, player_id: &str) -> Value {
    let clean_path = if path.starts_with("state.") {
        &path[6..] // Remove "state." prefix
    } else if path.starts_with("/") {
        &path[1..] // Remove leading "/"
    } else {
        path
    };
    
    // Replace player placeholders
    let resolved_path = clean_path.replace("{player}", player_id);
    
    // Navigate the state using the path
    let parts: Vec<&str> = resolved_path.split('.').collect();
    let mut current = game_state;
    
    for part in parts {
        if let Some(next) = current.get(part) {
            current = next;
        } else {
            return json!(null);
        }
    }
    
    current.clone()
}

/// Evaluate simple mathematical expressions
fn evaluate_math_expression(expression: &str, game_state: &Value, player_id: &str) -> Value {
    // Support basic math operations: +, -, *, /, %
    // First, try to parse as a simple binary operation
    
    // Replace state references in the expression
    let mut resolved_expr = expression.to_string();
    
    // Find and replace state path references
    let state_ref_regex = Regex::new(r"state\.[a-zA-Z0-9_.{}]+").unwrap();
    for capture in state_ref_regex.captures_iter(expression) {
        if let Some(state_ref) = capture.get(0) {
            let state_path = state_ref.as_str();
            let value = evaluate_state_path(state_path, game_state, player_id);
            if let Some(num) = value.as_i64() {
                resolved_expr = resolved_expr.replace(state_path, &num.to_string());
            } else if let Some(num) = value.as_f64() {
                resolved_expr = resolved_expr.replace(state_path, &num.to_string());
            }
        }
    }
    
    // Find and replace count() expressions
    let count_regex = Regex::new(r"count\([^)]+\)").unwrap();
    for capture in count_regex.captures_iter(&resolved_expr.clone()) {
        if let Some(count_expr) = capture.get(0) {
            let expr_str = count_expr.as_str();
            let zone_path = &expr_str[6..expr_str.len()-1];
            let count_val = evaluate_count_expression(zone_path, game_state);
            if let Some(num) = count_val.as_i64() {
                resolved_expr = resolved_expr.replace(expr_str, &num.to_string());
            }
        }
    }
    
    // Try to evaluate simple arithmetic
    if let Some(result) = evaluate_simple_arithmetic(&resolved_expr) {
        json!(result)
    } else {
        // Return the resolved expression if we can't evaluate it
        json!(resolved_expr)
    }
}

/// Evaluate simple arithmetic expressions
fn evaluate_simple_arithmetic(expr: &str) -> Option<f64> {
    // Remove whitespace
    let expr = expr.replace(" ", "");
    
    // Try division
    if let Some(pos) = expr.find('/') {
        let (left, right) = expr.split_at(pos);
        let right = &right[1..];
        if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
            if r != 0.0 {
                return Some(l / r);
            }
        }
    }
    
    // Try multiplication  
    if let Some(pos) = expr.find('*') {
        let (left, right) = expr.split_at(pos);
        let right = &right[1..];
        if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
            return Some(l * r);
        }
    }
    
    // Try addition
    if let Some(pos) = expr.find('+') {
        let (left, right) = expr.split_at(pos);
        let right = &right[1..];
        if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
            return Some(l + r);
        }
    }
    
    // Try subtraction (be careful not to match negative numbers)
    if let Some(pos) = expr[1..].find('-').map(|p| p + 1) {
        let (left, right) = expr.split_at(pos);
        let right = &right[1..];
        if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
            return Some(l - r);
        }
    }
    
    // Try parsing as a single number
    expr.parse::<f64>().ok()
}

/// Determine view format based on type and definition
fn determine_view_format(view_type: &str, zone_def: &Value) -> Option<ViewFormat> {
    let style = match view_type {
        "strategic" => ViewStyle::Table,
        "log" => ViewStyle::Log,
        "progress" => ViewStyle::Cards,
        "summary" => ViewStyle::Summary,
        _ => ViewStyle::Table,
    };
    
    let max_entries = if view_type == "log" {
        zone_def.get("viewData")
            .and_then(|v| v.get("maxEntries"))
            .and_then(|m| m.as_u64())
            .map(|m| m as usize)
            .or(Some(10))
    } else {
        None
    };
    
    Some(ViewFormat {
        style,
        sort_by: None,
        show_delta: None,
        max_entries,
    })
}

/// Create common view zone presets
pub mod presets {
    use super::*;
    
    /// Score tracking view preset
    pub fn score_view() -> Value {
        json!({
            "shape": "view",
            "viewType": "strategic",
            "tier": "strategic",
            "viewData": [
                {
                    "field": "score",
                    "label": "Score",
                    "source": "state.scores.{player}",
                    "perPlayer": true
                },
                {
                    "field": "position",
                    "label": "Position", 
                    "source": "state.positions.{player}",
                    "perPlayer": true
                }
            ]
        })
    }
    
    /// Card count view preset
    pub fn card_count_view() -> Value {
        json!({
            "shape": "view",
            "viewType": "strategic",
            "tier": "strategic",
            "viewData": [
                {
                    "field": "handSize",
                    "label": "Cards in Hand",
                    "source": "count(hand_{player})",
                    "perPlayer": true
                },
                {
                    "field": "deckRemaining",
                    "label": "Deck",
                    "source": "count(deck)"
                }
            ]
        })
    }
    
    /// Game log view preset
    pub fn game_log_view() -> Value {
        json!({
            "shape": "view",
            "viewType": "log",
            "tier": "ambient",
            "viewData": {
                "maxEntries": 10,
                "source": "gameLog",
                "filter": "action"
            }
        })
    }
    
    /// Resource tracking view preset
    pub fn resource_view() -> Value {
        json!({
            "shape": "view",
            "viewType": "strategic", 
            "tier": "strategic",
            "viewData": [
                {
                    "field": "gold",
                    "label": "Gold",
                    "source": "state.resources.{player}.gold",
                    "perPlayer": true
                },
                {
                    "field": "wood",
                    "label": "Wood",
                    "source": "state.resources.{player}.wood",
                    "perPlayer": true
                },
                {
                    "field": "stone",
                    "label": "Stone",
                    "source": "state.resources.{player}.stone",
                    "perPlayer": true
                }
            ]
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_count_expression() {
        let game_state = json!({
            "zones": {
                "hand_p1": ["card1", "card2", "card3"],
                "deck": {
                    "items": ["card4", "card5"]
                }
            }
        });
        
        assert_eq!(evaluate_count_expression("hand_p1", &game_state), json!(3));
        assert_eq!(evaluate_count_expression("deck", &game_state), json!(2));
        assert_eq!(evaluate_count_expression("nonexistent", &game_state), json!(0));
    }
    
    #[test]
    fn test_state_path_evaluation() {
        let game_state = json!({
            "scores": {
                "p1": 10,
                "p2": 15
            },
            "currentRound": 3
        });
        
        assert_eq!(
            evaluate_state_path("state.scores.p1", &game_state, "p1"),
            json!(10)
        );
        assert_eq!(
            evaluate_state_path("currentRound", &game_state, "p1"),
            json!(3)
        );
    }
    
    #[test]
    fn test_sum_expression() {
        let game_state = json!({
            "zones": {
                "treasury_p1": [
                    {"id": "coin1", "value": 5},
                    {"id": "coin2", "value": 10},
                    {"id": "coin3", "value": 3}
                ]
            }
        });
        
        assert_eq!(evaluate_sum_expression("treasury_p1.value", &game_state), json!(18));
    }
    
    #[test]
    fn test_avg_expression() {
        let game_state = json!({
            "zones": {
                "hand_p1": [
                    {"id": "card1", "rank": 5},
                    {"id": "card2", "rank": 10},
                    {"id": "card3", "rank": 3}
                ]
            }
        });
        
        assert_eq!(evaluate_avg_expression("hand_p1.rank", &game_state), json!(6.0));
    }
    
    #[test]
    fn test_max_min_expressions() {
        let game_state = json!({
            "zones": {
                "scores": {
                    "items": [
                        {"player": "p1", "points": 25},
                        {"player": "p2", "points": 30},
                        {"player": "p3", "points": 20}
                    ]
                }
            }
        });
        
        assert_eq!(evaluate_max_expression("scores.points", &game_state), json!(30.0));
        assert_eq!(evaluate_min_expression("scores.points", &game_state), json!(20.0));
    }
    
    #[test]
    fn test_math_expression() {
        let game_state = json!({
            "scores": {
                "p1": 10,
                "p2": 15
            },
            "bonus": 5
        });
        
        // Test simple arithmetic
        assert_eq!(evaluate_simple_arithmetic("10+5"), Some(15.0));
        assert_eq!(evaluate_simple_arithmetic("20-8"), Some(12.0));
        assert_eq!(evaluate_simple_arithmetic("4*3"), Some(12.0));
        assert_eq!(evaluate_simple_arithmetic("15/3"), Some(5.0));
        
        // Test with state references
        let result = evaluate_math_expression("state.scores.p1 + state.bonus", &game_state, "p1");
        assert_eq!(result, json!(15.0));
    }
    
    #[test]
    fn test_view_zone_computation() {
        let bundle = crate::bundle::Bundle {
            game_id: "test-game".to_string(),
            manifest: crate::bundle::Manifest {
                game_id: "test-game".to_string(),
                version: "1.0".to_string(),
                spec_version: "1.0".to_string(),
                metadata: crate::bundle::ManifestMetadata {
                    name: "Test Game".to_string(),
                    author: "Test".to_string(),
                    players: crate::bundle::PlayersRange { min: 2, max: 4 },
                    description: "Test".to_string(),
                },
                phases: None,
                setup: None,
                zone_groups: None,
            },
            entities: json!({}),
            zones: json!({}),
            actions: json!({}),
            phases: json!({}),
        };
        
        let zone_def = json!({
            "id": "game_stats",
            "shape": "view",
            "viewType": "strategic",
            "viewData": [
                {
                    "field": "handSize",
                    "label": "Cards in Hand",
                    "source": "count(hand_{player})",
                    "perPlayer": true
                },
                {
                    "field": "score",
                    "label": "Score",
                    "source": "state.scores.{player}",
                    "perPlayer": true
                },
                {
                    "field": "deckRemaining",
                    "label": "Deck",
                    "source": "count(deck)"
                }
            ]
        });
        
        let game_state = json!({
            "zones": {
                "hand_p1": ["card1", "card2", "card3"],
                "hand_p2": ["card4", "card5"],
                "deck": ["card6", "card7", "card8", "card9"]
            },
            "scores": {
                "p1": 10,
                "p2": 15
            }
        });
        
        let player_names = vec!["Alice".to_string(), "Bob".to_string()];
        
        let view_data = compute_view_zone_data(
            "game_stats",
            &zone_def,
            &game_state,
            &player_names,
            &bundle
        );
        
        assert_eq!(view_data.view_type, "strategic");
        assert_eq!(view_data.data.players["p1"]["handSize"], json!(3));
        assert_eq!(view_data.data.players["p2"]["handSize"], json!(2));
        assert_eq!(view_data.data.players["p1"]["score"], json!(10));
        assert_eq!(view_data.data.players["p2"]["score"], json!(15));
        assert_eq!(view_data.data.shared["deckRemaining"], json!(4));
    }
}