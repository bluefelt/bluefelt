//! Hex-specific win detection verbs for games like hex tic-tac-toe

use serde_json::{json, Value};
use crate::bundle::Bundle;
use crate::engine::hex::{HexCoord, HexDirection};

/// Check if a player has won on a hex grid by forming a line
/// Args:
/// - zone: The hex grid zone to check
/// - player: The player to check for
/// - lineLength: The required line length to win (default: 3)
/// - lastMove: The last move location (optional, for optimization)
pub fn apply_check_hex_win(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("checkHexWin requires 'zone' parameter")?;
    
    let player = args.get("player")
        .and_then(|p| p.as_str())
        .ok_or("checkHexWin requires 'player' parameter")?;
    
    let line_length = args.get("lineLength")
        .and_then(|l| l.as_u64())
        .unwrap_or(3) as usize;
    
    // Get the zone data
    let zone = get_zone_from_path(state, zone_path)?;
    
    // Check if this is a hex grid
    let zone_type = zone.get("type")
        .and_then(|t| t.as_str())
        .ok_or("Zone missing 'type' field")?;
    
    if zone_type != "hexgrid" {
        return Err(format!("checkHexWin only works on hexgrid zones, found: {}", zone_type));
    }
    
    // Get all cells with the player's marks
    let player_cells = find_player_cells(zone, player)?;
    
    // If we have a lastMove hint, start checking from there
    if let Some(last_move) = args.get("lastMove").and_then(|m| m.as_str()) {
        if let Ok(last_coord) = HexCoord::from_string(last_move) {
            if check_win_from_position(&player_cells, last_coord, line_length) {
                return Ok(vec![json!({
                    "op": "add",
                    "path": "/meta/winner",
                    "value": player
                })]);
            }
        }
    }
    
    // Otherwise check all player positions
    for coord in &player_cells {
        if check_win_from_position(&player_cells, *coord, line_length) {
            return Ok(vec![json!({
                "op": "add",
                "path": "/meta/winner",
                "value": player
            })]);
        }
    }
    
    // No win found
    Ok(vec![])
}

/// Check for a line of specific pattern on hex grid
/// Args:
/// - zone: The hex grid zone to check
/// - pattern: The pattern to match (e.g., "mark_p1")
/// - direction: Optional specific direction to check (0-5 for hex directions)
/// - startCoord: Starting coordinate to check from
/// - length: Required line length
pub fn apply_check_hex_line(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("checkHexLine requires 'zone' parameter")?;
    
    let pattern = args.get("pattern")
        .and_then(|p| p.as_str())
        .ok_or("checkHexLine requires 'pattern' parameter")?;
    
    let length = args.get("length")
        .and_then(|l| l.as_u64())
        .unwrap_or(3) as usize;
    
    let zone = get_zone_from_path(state, zone_path)?;
    
    // Get cells matching pattern
    let matching_cells = find_pattern_cells(zone, pattern)?;
    
    // Check if a specific starting coordinate is provided
    if let Some(start_str) = args.get("startCoord").and_then(|s| s.as_str()) {
        let start_coord = HexCoord::from_string(start_str)?;
        
        // Check specific direction if provided
        if let Some(dir_num) = args.get("direction").and_then(|d| d.as_u64()) {
            if let Some(direction) = HexDirection::from_int(dir_num as usize) {
                let has_line = check_line_in_direction(&matching_cells, start_coord, direction, length);
                return Ok(vec![json!({
                    "op": "add",
                    "path": "/temp/hasLine",
                    "value": has_line
                })]);
            }
        }
        
        // Check all directions from start
        for direction in HexDirection::all() {
            if check_line_in_direction(&matching_cells, start_coord, direction, length) {
                return Ok(vec![json!({
                    "op": "add",
                    "path": "/temp/hasLine",
                    "value": true
                })]);
            }
        }
    }
    
    // Check all positions for any line
    for coord in &matching_cells {
        for direction in HexDirection::all() {
            if check_line_in_direction(&matching_cells, *coord, direction, length) {
                return Ok(vec![json!({
                    "op": "add",
                    "path": "/temp/hasLine",
                    "value": true
                })]);
            }
        }
    }
    
    Ok(vec![json!({
        "op": "add",
        "path": "/temp/hasLine",
        "value": false
    })])
}

/// Count connected groups on hex grid
/// Args:
/// - zone: The hex grid zone to check
/// - pattern: The pattern to match for grouping
pub fn apply_count_hex_groups(state: &mut Value, args: &Value) -> Result<Vec<Value>, String> {
    let zone_path = args.get("zone")
        .and_then(|z| z.as_str())
        .ok_or("countHexGroups requires 'zone' parameter")?;
    
    let pattern = args.get("pattern")
        .and_then(|p| p.as_str())
        .ok_or("countHexGroups requires 'pattern' parameter")?;
    
    let zone = get_zone_from_path(state, zone_path)?;
    let matching_cells = find_pattern_cells(zone, pattern)?;
    
    let group_count = count_connected_groups(&matching_cells);
    
    Ok(vec![json!({
        "op": "add",
        "path": "/temp/groupCount",
        "value": group_count
    })])
}

// Helper functions

fn get_zone_from_path<'a>(state: &'a Value, path: &str) -> Result<&'a Value, String> {
    let path_parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    
    let mut current = state;
    for part in path_parts {
        current = current.get(part)
            .ok_or(format!("Invalid path: {}", path))?;
    }
    
    Ok(current)
}

fn find_player_cells(zone: &Value, player: &str) -> Result<Vec<HexCoord>, String> {
    let cells = zone.get("cells")
        .and_then(|c| c.as_object())
        .ok_or("Zone missing 'cells' field")?;
    
    let player_entity = format!("mark_{}", player);
    let mut player_cells = Vec::new();
    
    for (coord_str, cell) in cells {
        if let Some(content) = cell.get("contents") {
            if content.get("entity").and_then(|e| e.as_str()) == Some(&player_entity) {
                if let Ok(coord) = HexCoord::from_string(coord_str) {
                    player_cells.push(coord);
                }
            }
        }
    }
    
    Ok(player_cells)
}

fn find_pattern_cells(zone: &Value, pattern: &str) -> Result<Vec<HexCoord>, String> {
    let cells = zone.get("cells")
        .and_then(|c| c.as_object())
        .ok_or("Zone missing 'cells' field")?;
    
    let mut matching_cells = Vec::new();
    
    for (coord_str, cell) in cells {
        if let Some(content) = cell.get("contents") {
            if content.get("entity").and_then(|e| e.as_str()) == Some(pattern) {
                if let Ok(coord) = HexCoord::from_string(coord_str) {
                    matching_cells.push(coord);
                }
            }
        }
    }
    
    Ok(matching_cells)
}

fn check_win_from_position(player_cells: &[HexCoord], start: HexCoord, line_length: usize) -> bool {
    // Check all 6 directions from this position
    for direction in HexDirection::all() {
        if check_line_in_direction(player_cells, start, direction, line_length) {
            return true;
        }
    }
    
    // Also check lines that pass through this position (not just starting from it)
    for direction in [HexDirection::East, HexDirection::Southeast, HexDirection::Southwest] {
        let opposite = direction.opposite();
        
        // Count in both directions from this position
        let forward_count = count_in_direction(player_cells, start, direction, line_length);
        let backward_count = count_in_direction(player_cells, start, opposite, line_length);
        
        // Total line length is forward + backward + 1 (the starting position)
        if forward_count + backward_count + 1 >= line_length {
            return true;
        }
    }
    
    false
}

fn check_line_in_direction(cells: &[HexCoord], start: HexCoord, direction: HexDirection, length: usize) -> bool {
    if !cells.contains(&start) {
        return false;
    }
    
    let mut count = 1;
    let mut current = start;
    
    // Get direction offset
    let offset = get_direction_offset(direction);
    
    // Check in the given direction
    for _ in 1..length {
        current = current + offset;
        if !cells.contains(&current) {
            return false;
        }
        count += 1;
    }
    
    count >= length
}

fn count_in_direction(cells: &[HexCoord], start: HexCoord, direction: HexDirection, max_count: usize) -> usize {
    let mut count = 0;
    let mut current = start;
    let offset = get_direction_offset(direction);
    
    // Don't count the starting position
    for _ in 0..max_count {
        current = current + offset;
        if !cells.contains(&current) {
            break;
        }
        count += 1;
    }
    
    count
}

fn get_direction_offset(direction: HexDirection) -> HexCoord {
    match direction {
        HexDirection::East => HexCoord::new(1, 0),
        HexDirection::Southeast => HexCoord::new(0, 1),
        HexDirection::Southwest => HexCoord::new(-1, 1),
        HexDirection::West => HexCoord::new(-1, 0),
        HexDirection::Northwest => HexCoord::new(0, -1),
        HexDirection::Northeast => HexCoord::new(1, -1),
    }
}

fn count_connected_groups(cells: &[HexCoord]) -> usize {
    use std::collections::HashSet;
    
    let mut visited = HashSet::new();
    let mut group_count = 0;
    let cell_set: HashSet<_> = cells.iter().copied().collect();
    
    for &coord in cells {
        if !visited.contains(&coord) {
            // Start a new group
            group_count += 1;
            
            // Flood fill to mark all connected cells
            let mut stack = vec![coord];
            while let Some(current) = stack.pop() {
                if visited.insert(current) {
                    // Check all neighbors
                    for direction in HexDirection::all() {
                        let neighbor = current + get_direction_offset(direction);
                        if cell_set.contains(&neighbor) && !visited.contains(&neighbor) {
                            stack.push(neighbor);
                        }
                    }
                }
            }
        }
    }
    
    group_count
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_direction_offsets() {
        let east = get_direction_offset(HexDirection::East);
        assert_eq!(east, HexCoord::new(1, 0));
        
        let northwest = get_direction_offset(HexDirection::Northwest);
        assert_eq!(northwest, HexCoord::new(0, -1));
    }
    
    #[test]
    fn test_line_detection() {
        let cells = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        
        assert!(check_line_in_direction(&cells, HexCoord::new(0, 0), HexDirection::East, 3));
        assert!(!check_line_in_direction(&cells, HexCoord::new(0, 0), HexDirection::East, 4));
        assert!(!check_line_in_direction(&cells, HexCoord::new(0, 0), HexDirection::Southeast, 3));
    }
    
    #[test]
    fn test_win_detection_through_position() {
        let cells = vec![
            HexCoord::new(-1, 0),
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
        ];
        
        // Should detect win from middle position
        assert!(check_win_from_position(&cells, HexCoord::new(0, 0), 3));
        // Should also detect from end positions
        assert!(check_win_from_position(&cells, HexCoord::new(-1, 0), 3));
        assert!(check_win_from_position(&cells, HexCoord::new(1, 0), 3));
    }
    
    #[test]
    fn test_diagonal_win() {
        let cells = vec![
            HexCoord::new(0, 0),
            HexCoord::new(0, 1),
            HexCoord::new(0, 2),
        ];
        
        // Southeast diagonal - correct hex coordinates
        assert!(check_win_from_position(&cells, HexCoord::new(0, 0), 3));
    }
}