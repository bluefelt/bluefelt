//! YAML shortcuts and syntactic sugar for more readable game definitions

use serde_yaml::Value as YamlValue;
use serde_yaml::Mapping;
use anyhow::Result;

/// Expand YAML shortcuts into their full form
pub fn expand_shortcuts(yaml: YamlValue) -> Result<YamlValue> {
    match yaml {
        YamlValue::Mapping(map) => expand_mapping_shortcuts(map),
        YamlValue::Sequence(seq) => {
            // Check if this is an entities sequence and expand deck shortcuts
            let mut expanded_seq = Vec::new();
            for item in seq {
                if let YamlValue::Mapping(ref map) = item {
                    // Check for deck shorthand
                    if map.contains_key(&YamlValue::String("deck".to_string())) {
                        // Handle deck shorthand
                        match map.get(&YamlValue::String("deck".to_string())) {
                            Some(YamlValue::String(deck_type)) if deck_type == "standard-52" => {
                                // Expand standard deck into multiple card entities
                                let deck_entities = create_standard_deck();
                                if let YamlValue::Sequence(cards) = deck_entities {
                                    expanded_seq.extend(cards);
                                }
                            }
                            Some(YamlValue::Mapping(deck_config)) => {
                                // Enhanced deck with configuration
                                let deck_entities = create_configured_deck(deck_config)?;
                                if let YamlValue::Sequence(cards) = deck_entities {
                                    expanded_seq.extend(cards);
                                }
                            }
                            _ => {
                                // Unknown deck format, just pass through
                                expanded_seq.push(expand_shortcuts(item)?);
                            }
                        }
                    } else {
                        // Not a deck shorthand, recursively expand
                        expanded_seq.push(expand_shortcuts(item)?);
                    }
                } else {
                    expanded_seq.push(expand_shortcuts(item)?);
                }
            }
            Ok(YamlValue::Sequence(expanded_seq))
        }
        _ => Ok(yaml),
    }
}

fn expand_mapping_shortcuts(mut map: Mapping) -> Result<YamlValue> {
    // First check if this mapping is an action (has "uses" or "id" field)
    let is_action = map.contains_key(&YamlValue::String("uses".to_string())) ||
                    map.contains_key(&YamlValue::String("id".to_string())) &&
                    (map.contains_key(&YamlValue::String("if".to_string())) ||
                     map.contains_key(&YamlValue::String("when".to_string())));
    
    if is_action {
        expand_action_shortcuts(&mut map)?;
    }
    
    let mut expanded = Mapping::new();
    
    for (key, value) in map.into_iter() {
        let key_str = key.as_str().unwrap_or("");
        
        match key_str {
            // Action shortcuts
            "turn" => {
                // Shortcut: turn: true → is_turn_based: true
                if value.as_bool() == Some(true) {
                    expanded.insert(YamlValue::String("is_turn_based".to_string()), YamlValue::Bool(true));
                }
            }
            
            // Entity shortcuts
            "deck" => {
                // Shortcut for card deck definitions
                match &value {
                    YamlValue::String(s) if s == "standard-52" => {
                        expanded.insert(YamlValue::String("entities".to_string()), 
                            create_standard_deck());
                    }
                    YamlValue::Mapping(deck_config) => {
                        // Enhanced deck with configuration
                        expanded.insert(YamlValue::String("entities".to_string()), 
                            create_configured_deck(deck_config)?);
                    }
                    _ => {
                        expanded.insert(key, value);
                    }
                }
            }
            
            // Zone shortcuts
            "grid" => {
                // Shortcut: grid: 3x3 → expands to board zone with cells
                if let Some(grid_str) = value.as_str() {
                    if let Some((rows, cols)) = parse_grid_size(grid_str) {
                        expanded.insert(YamlValue::String("type".to_string()), 
                            YamlValue::String("grid".to_string()));
                        
                        let mut grid_props = Mapping::new();
                        grid_props.insert(YamlValue::String("rows".to_string()), 
                            YamlValue::Number(rows.into()));
                        grid_props.insert(YamlValue::String("cols".to_string()), 
                            YamlValue::Number(cols.into()));
                        
                        expanded.insert(YamlValue::String("gridProps".to_string()), 
                            YamlValue::Mapping(grid_props));
                    }
                }
            }
            
            // Phase shortcuts
            "win" => {
                // Shortcut for win conditions
                // win: "3-in-a-row" → expands to full condition
                if let Some(win_type) = value.as_str() {
                    expanded.insert(YamlValue::String("onEnter".to_string()), 
                        create_win_condition(win_type)?);
                }
            }
            
            // Condition shortcuts
            "empty" => {
                // Shortcut: empty: true → condition: zone.isEmpty
                if value.as_bool() == Some(true) {
                    let condition = create_condition("zone.isEmpty", Mapping::new());
                    expanded.insert(YamlValue::String("condition".to_string()), condition);
                }
            }
            
            "owned_by" => {
                // Shortcut: owned_by: "{player}" → condition: entity.owner with: owner: "{player}"
                if let Some(owner) = value.as_str() {
                    let mut with_map = Mapping::new();
                    with_map.insert(YamlValue::String("owner".to_string()), 
                        YamlValue::String(owner.to_string()));
                    let condition = create_condition("entity.owner", with_map);
                    expanded.insert(YamlValue::String("condition".to_string()), condition);
                }
            }
            
            // Recursively expand nested values
            _ => {
                let expanded_value = expand_shortcuts(value)?;
                expanded.insert(key, expanded_value);
            }
        }
    }
    
    Ok(YamlValue::Mapping(expanded))
}

fn parse_grid_size(grid_str: &str) -> Option<(i64, i64)> {
    let parts: Vec<&str> = grid_str.split('x').collect();
    if parts.len() == 2 {
        let rows = parts[0].parse::<i64>().ok()?;
        let cols = parts[1].parse::<i64>().ok()?;
        Some((rows, cols))
    } else {
        None
    }
}

fn create_standard_deck() -> YamlValue {
    let suits = vec!["hearts", "diamonds", "clubs", "spades"];
    let ranks = vec!["a", "2", "3", "4", "5", "6", "7", "8", "9", "10", "j", "q", "k"];
    
    let mut entities = Vec::new();
    for suit in &suits {
        for rank in &ranks {
            let mut card = Mapping::new();
            card.insert(YamlValue::String("id".to_string()), 
                YamlValue::String(format!("card_{}_{}", rank, suit)));
            card.insert(YamlValue::String("type".to_string()), 
                YamlValue::String("card".to_string()));
            
            let mut properties = Mapping::new();
            properties.insert(YamlValue::String("rank".to_string()), 
                YamlValue::String(rank.to_string()));
            properties.insert(YamlValue::String("suit".to_string()), 
                YamlValue::String(suit.to_string()));
            
            card.insert(YamlValue::String("props".to_string()), 
                YamlValue::Mapping(properties));
            
            entities.push(YamlValue::Mapping(card));
        }
    }
    
    YamlValue::Sequence(entities)
}

/// Create a configured deck based on bf.standardDeck with overrides
fn create_configured_deck(config: &Mapping) -> Result<YamlValue> {
    let mut suits: Vec<String> = vec!["hearts", "diamonds", "clubs", "spades"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let mut ranks: Vec<String> = vec!["a", "2", "3", "4", "5", "6", "7", "8", "9", "10", "j", "q", "k"]
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let mut copies = 1;
    let mut include_jokers = false;
    let mut joker_count = 2;
    let mut rank_values: Option<&Mapping> = None;
    let mut rank_names: Option<&Mapping> = None;
    let mut suit_colors: Option<&Mapping> = None;
    let mut id_pattern = "card_{rank}_{suit}".to_string();
    let mut ace_high = false;
    let mut wild_cards: Vec<String> = Vec::new();
    
    // Check for bf.standardDeck base
    if let Some(YamlValue::String(base)) = config.get(&YamlValue::String("as".to_string())) {
        if base != "bf.standardDeck" {
            return Err(anyhow::anyhow!("Unknown deck type: {}", base));
        }
    } else {
        return Err(anyhow::anyhow!("Deck configuration must specify 'as: bf.standardDeck'"));
    }
    
    // Apply overrides from 'with' section
    if let Some(YamlValue::Mapping(overrides)) = config.get(&YamlValue::String("with".to_string())) {
        // Custom suits
        if let Some(YamlValue::Sequence(custom_suits)) = overrides.get(&YamlValue::String("suits".to_string())) {
            suits = custom_suits.iter()
                .filter_map(|s| s.as_str())
                .map(|s| s.to_string())
                .collect();
        }
        
        // Custom ranks
        if let Some(YamlValue::Sequence(custom_ranks)) = overrides.get(&YamlValue::String("ranks".to_string())) {
            ranks = custom_ranks.iter()
                .filter_map(|r| r.as_str())
                .map(|r| r.to_string())
                .collect();
        }
        
        // Number of copies
        if let Some(YamlValue::Number(n)) = overrides.get(&YamlValue::String("copies".to_string())) {
            copies = n.as_i64().unwrap_or(1) as i32;
        }
        
        // Jokers
        if let Some(YamlValue::Bool(b)) = overrides.get(&YamlValue::String("includeJokers".to_string())) {
            include_jokers = *b;
        }
        if let Some(YamlValue::Number(n)) = overrides.get(&YamlValue::String("jokerCount".to_string())) {
            joker_count = n.as_i64().unwrap_or(2) as i32;
        }
        
        // Rank values and names
        if let Some(YamlValue::Mapping(values)) = overrides.get(&YamlValue::String("rankValues".to_string())) {
            rank_values = Some(values);
        }
        if let Some(YamlValue::Mapping(names)) = overrides.get(&YamlValue::String("rankNames".to_string())) {
            rank_names = Some(names);
        }
        if let Some(YamlValue::Mapping(colors)) = overrides.get(&YamlValue::String("suitColors".to_string())) {
            suit_colors = Some(colors);
        }
        
        // ID pattern
        if let Some(YamlValue::String(pattern)) = overrides.get(&YamlValue::String("idPattern".to_string())) {
            id_pattern = pattern.clone();
        }
        
        // Ace high/low
        if let Some(YamlValue::Bool(b)) = overrides.get(&YamlValue::String("aceHigh".to_string())) {
            ace_high = *b;
        }
        
        // Wild cards
        if let Some(YamlValue::Sequence(wilds)) = overrides.get(&YamlValue::String("wildCards".to_string())) {
            wild_cards = wilds.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }
    }
    
    let mut entities = Vec::new();
    
    // Generate cards
    for copy in 0..copies {
        for suit in &suits {
            for rank in &ranks {
                let mut card = Mapping::new();
                
                // Generate ID based on pattern
                let id = if copies > 1 {
                    id_pattern
                        .replace("{rank}", rank)
                        .replace("{suit}", suit)
                        .replace("{copy}", &(copy + 1).to_string())
                } else {
                    id_pattern
                        .replace("{rank}", rank)
                        .replace("{suit}", suit)
                };
                
                card.insert(YamlValue::String("id".to_string()), 
                    YamlValue::String(id));
                card.insert(YamlValue::String("type".to_string()), 
                    YamlValue::String("card".to_string()));
                
                let mut properties = Mapping::new();
                properties.insert(YamlValue::String("rank".to_string()), 
                    YamlValue::String(rank.to_string()));
                properties.insert(YamlValue::String("suit".to_string()), 
                    YamlValue::String(suit.to_string()));
                
                // Add rank value if specified
                if let Some(values) = rank_values {
                    if let Some(value) = values.get(&YamlValue::String(rank.to_string())) {
                        properties.insert(YamlValue::String("value".to_string()), value.clone());
                    }
                } else if rank == "a" && ace_high {
                    // Default ace high value
                    properties.insert(YamlValue::String("value".to_string()), YamlValue::Number(14.into()));
                }
                
                // Check if this card is wild
                let is_wild = wild_cards.contains(rank) || wild_cards.contains(&format!("{}_{}", rank, suit));
                if is_wild {
                    properties.insert(YamlValue::String("isWild".to_string()), YamlValue::Bool(true));
                }
                
                // Add rank display name if specified
                if let Some(names) = rank_names {
                    if let Some(name) = names.get(&YamlValue::String(rank.to_string())) {
                        properties.insert(YamlValue::String("displayName".to_string()), name.clone());
                    }
                }
                
                // Add suit color if specified
                if let Some(colors) = suit_colors {
                    if let Some(color) = colors.get(&YamlValue::String(suit.to_string())) {
                        properties.insert(YamlValue::String("color".to_string()), color.clone());
                    }
                }
                
                // Add default display properties
                let rank_display = rank_names
                    .and_then(|names| names.get(&YamlValue::String(rank.to_string())))
                    .and_then(|v| v.as_str())
                    .unwrap_or(rank)
                    .to_uppercase();
                
                let suit_symbol = match suit.as_str() {
                    "hearts" => "♥",
                    "diamonds" => "♦", 
                    "clubs" => "♣",
                    "spades" => "♠",
                    _ => ""
                };
                
                properties.insert(YamlValue::String("displayRank".to_string()), 
                    YamlValue::String(rank_display));
                properties.insert(YamlValue::String("displaySuit".to_string()), 
                    YamlValue::String(suit_symbol.to_string()));
                
                card.insert(YamlValue::String("props".to_string()), 
                    YamlValue::Mapping(properties));
                
                entities.push(YamlValue::Mapping(card));
            }
        }
    }
    
    // Add jokers if requested
    if include_jokers {
        for i in 0..joker_count {
            let mut joker = Mapping::new();
            joker.insert(YamlValue::String("id".to_string()), 
                YamlValue::String(format!("joker_{}", i + 1)));
            joker.insert(YamlValue::String("type".to_string()), 
                YamlValue::String("card".to_string()));
            
            let mut properties = Mapping::new();
            properties.insert(YamlValue::String("rank".to_string()), 
                YamlValue::String("joker".to_string()));
            properties.insert(YamlValue::String("suit".to_string()), 
                YamlValue::String("none".to_string()));
            properties.insert(YamlValue::String("isJoker".to_string()), 
                YamlValue::Bool(true));
            
            joker.insert(YamlValue::String("props".to_string()), 
                YamlValue::Mapping(properties));
            
            entities.push(YamlValue::Mapping(joker));
        }
    }
    
    Ok(YamlValue::Sequence(entities))
}

fn create_win_condition(win_type: &str) -> Result<YamlValue> {
    match win_type {
        "3-in-a-row" => {
            let mut action = Mapping::new();
            action.insert(YamlValue::String("uses".to_string()), 
                YamlValue::String("calculateWinner".to_string()));
            
            let mut with_map = Mapping::new();
            with_map.insert(YamlValue::String("type".to_string()), 
                YamlValue::String("line".to_string()));
            with_map.insert(YamlValue::String("length".to_string()), 
                YamlValue::Number(3.into()));
            
            action.insert(YamlValue::String("with".to_string()), 
                YamlValue::Mapping(with_map));
            
            Ok(YamlValue::Sequence(vec![YamlValue::Mapping(action)]))
        }
        _ => Err(anyhow::anyhow!("Unknown win condition type: {}", win_type))
    }
}

fn create_condition(condition_type: &str, with_map: Mapping) -> YamlValue {
    let mut condition = Mapping::new();
    condition.insert(YamlValue::String("condition".to_string()), 
        YamlValue::String(condition_type.to_string()));
    
    if !with_map.is_empty() {
        condition.insert(YamlValue::String("with".to_string()), 
            YamlValue::Mapping(with_map));
    }
    
    YamlValue::Mapping(condition)
}

/// Expand if/then/else shorthand for conditions
fn expand_if_then_else(mapping: &mut Mapping) -> Result<()> {
    // Check for if: field
    if let Some(if_value) = mapping.remove(&YamlValue::String("if".to_string())) {
        // Convert the if: value into a when: array
        let when_conditions = match if_value {
            YamlValue::String(condition_str) => {
                // Simple string condition like if: "zone.isEmpty"
                vec![create_simple_condition(&condition_str)]
            }
            YamlValue::Mapping(condition_map) => {
                // Object condition like if: { phase: "main" }
                vec![expand_condition_object(condition_map)?]
            }
            YamlValue::Sequence(conditions) => {
                // Array of conditions - convert each one
                conditions.into_iter()
                    .map(|c| match c {
                        YamlValue::String(s) => Ok(create_simple_condition(&s)),
                        YamlValue::Mapping(m) => expand_condition_object(m),
                        _ => Err(anyhow::anyhow!("Invalid condition format")),
                    })
                    .collect::<Result<Vec<_>>>()?
            }
            _ => return Err(anyhow::anyhow!("Invalid if: format")),
        };
        
        // Add the when: array to the mapping
        mapping.insert(
            YamlValue::String("when".to_string()), 
            YamlValue::Sequence(when_conditions)
        );
    }
    
    // Handle else: by converting to a separate action with negated conditions
    // This is more complex and would need engine support, so skip for now
    
    Ok(())
}

/// Create a simple condition from a string like "zone.isEmpty"
fn create_simple_condition(condition_str: &str) -> YamlValue {
    let mut condition = Mapping::new();
    condition.insert(
        YamlValue::String("condition".to_string()),
        YamlValue::String(condition_str.to_string())
    );
    YamlValue::Mapping(condition)
}

/// Expand a condition object shorthand
fn expand_condition_object(mut map: Mapping) -> Result<YamlValue> {
    // Handle common condition shortcuts
    if let Some(phase) = map.remove(&YamlValue::String("phase".to_string())) {
        // if: { phase: "main" } → condition: phase.is, with: { phase: "main" }
        let mut condition = Mapping::new();
        condition.insert(
            YamlValue::String("condition".to_string()),
            YamlValue::String("phase.is".to_string())
        );
        
        let mut with_map = Mapping::new();
        with_map.insert(YamlValue::String("phase".to_string()), phase);
        condition.insert(
            YamlValue::String("with".to_string()),
            YamlValue::Mapping(with_map)
        );
        
        return Ok(YamlValue::Mapping(condition));
    }
    
    if let Some(owner) = map.remove(&YamlValue::String("owner".to_string())) {
        // if: { owner: "{player}" } → condition: entity.owner, with: { owner: "{player}" }
        let mut condition = Mapping::new();
        condition.insert(
            YamlValue::String("condition".to_string()),
            YamlValue::String("entity.owner".to_string())
        );
        
        let mut with_map = Mapping::new();
        with_map.insert(YamlValue::String("owner".to_string()), owner);
        condition.insert(
            YamlValue::String("with".to_string()),
            YamlValue::Mapping(with_map)
        );
        
        return Ok(YamlValue::Mapping(condition));
    }
    
    if let Some(empty) = map.remove(&YamlValue::String("empty".to_string())) {
        // if: { empty: true } → condition: zone.isEmpty
        if empty.as_bool() == Some(true) {
            return Ok(create_simple_condition("zone.isEmpty"));
        } else {
            return Ok(create_simple_condition("zone.hasEntities"));
        }
    }
    
    // If we don't recognize the shorthand, treat it as a generic condition
    if map.len() == 1 {
        let (key, value) = map.into_iter().next().unwrap();
        if let Some(key_str) = key.as_str() {
            let mut condition = Mapping::new();
            condition.insert(
                YamlValue::String("condition".to_string()),
                YamlValue::String(key_str.to_string())
            );
            
            // Add the value as a parameter
            let mut with_map = Mapping::new();
            with_map.insert(YamlValue::String("value".to_string()), value);
            condition.insert(
                YamlValue::String("with".to_string()),
                YamlValue::Mapping(with_map)
            );
            
            return Ok(YamlValue::Mapping(condition));
        }
    }
    
    Err(anyhow::anyhow!("Unrecognized condition shorthand"))
}

/// Expand standard library actions like uses: "bf.deal"
fn expand_standard_library_actions(action: &mut Mapping) -> Result<()> {
    if let Some(uses) = action.get(&YamlValue::String("uses".to_string())) {
        if let Some(uses_str) = uses.as_str() {
            if uses_str.starts_with("bf.") {
                // This is a standard library action
                match uses_str {
                    "bf.deal" => expand_bf_deal(action)?,
                    "bf.draw" => expand_bf_draw(action)?,
                    "bf.shuffle" => expand_bf_shuffle(action)?,
                    "bf.nextTurn" => expand_bf_next_turn(action)?,
                    "bf.endGame" => expand_bf_end_game(action)?,
                    "bf.place" => expand_bf_place(action)?,
                    "bf.move" => expand_bf_move(action)?,
                    _ => return Err(anyhow::anyhow!("Unknown standard library action: {}", uses_str)),
                }
            }
        }
    }
    Ok(())
}

/// Expand bf.deal - deal cards from a deck to players
fn expand_bf_deal(action: &mut Mapping) -> Result<()> {
    // Replace uses: "bf.deal" with uses: "dealEntities"
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("dealEntities".to_string())
    );
    
    // Add default parameters if not specified
    if !action.contains_key(&YamlValue::String("with".to_string())) {
        let mut with_map = Mapping::new();
        with_map.insert(
            YamlValue::String("from".to_string()),
            YamlValue::String("deck".to_string())
        );
        with_map.insert(
            YamlValue::String("to".to_string()),
            YamlValue::String("hand_{player}".to_string())
        );
        with_map.insert(
            YamlValue::String("count".to_string()),
            YamlValue::Number(7.into())  // Default to 7 cards
        );
        action.insert(
            YamlValue::String("with".to_string()),
            YamlValue::Mapping(with_map)
        );
    }
    
    Ok(())
}

/// Expand bf.draw - draw cards from deck to hand
fn expand_bf_draw(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("transferEntity".to_string())
    );
    
    // Set default from/to if not specified
    let mut with_map = if let Some(YamlValue::Mapping(m)) = action.get(&YamlValue::String("with".to_string())) {
        m.clone()
    } else {
        Mapping::new()
    };
    
    if !with_map.contains_key(&YamlValue::String("from".to_string())) {
        with_map.insert(
            YamlValue::String("from".to_string()),
            YamlValue::String("deck".to_string())
        );
    }
    
    if !with_map.contains_key(&YamlValue::String("to".to_string())) {
        with_map.insert(
            YamlValue::String("to".to_string()),
            YamlValue::String("hand_{player}".to_string())
        );
    }
    
    action.insert(
        YamlValue::String("with".to_string()),
        YamlValue::Mapping(with_map)
    );
    
    Ok(())
}

/// Expand bf.shuffle - shuffle a zone
fn expand_bf_shuffle(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("shuffle".to_string())
    );
    
    // Default to shuffling the deck
    if !action.contains_key(&YamlValue::String("with".to_string())) {
        let mut with_map = Mapping::new();
        with_map.insert(
            YamlValue::String("zone".to_string()),
            YamlValue::String("deck".to_string())
        );
        action.insert(
            YamlValue::String("with".to_string()),
            YamlValue::Mapping(with_map)
        );
    }
    
    Ok(())
}

/// Expand bf.nextTurn - advance to next player's turn
fn expand_bf_next_turn(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("nextTurn".to_string())
    );
    Ok(())
}

/// Expand bf.endGame - end the game with a winner
fn expand_bf_end_game(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("endGame".to_string())
    );
    Ok(())
}

/// Expand bf.place - place an entity on the board
fn expand_bf_place(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("placeEntity".to_string())
    );
    Ok(())
}

/// Expand bf.move - move an entity between zones
fn expand_bf_move(action: &mut Mapping) -> Result<()> {
    action.insert(
        YamlValue::String("uses".to_string()),
        YamlValue::String("moveEntity".to_string())
    );
    Ok(())
}

/// Additional shortcuts for actions
pub fn expand_action_shortcuts(action: &mut Mapping) -> Result<()> {
    // First expand standard library actions
    expand_standard_library_actions(action)?;
    
    // Then expand if/then/else
    expand_if_then_else(action)?;
    
    // Shortcut: log: "message" → ui: { logTemplate: "message" }
    if let Some(log) = action.remove(&YamlValue::String("log".to_string())) {
        if let Some(log_str) = log.as_str() {
            let mut ui = Mapping::new();
            ui.insert(YamlValue::String("logTemplate".to_string()), 
                YamlValue::String(log_str.to_string()));
            action.insert(YamlValue::String("ui".to_string()), 
                YamlValue::Mapping(ui));
        }
    }
    
    // Shortcut: then: "nextTurn" → then: [{ uses: "nextTurn" }]
    if let Some(then) = action.get(&YamlValue::String("then".to_string())) {
        if let Some(then_str) = then.as_str() {
            let mut then_action = Mapping::new();
            then_action.insert(YamlValue::String("uses".to_string()), 
                YamlValue::String(then_str.to_string()));
            
            action.insert(YamlValue::String("then".to_string()), 
                YamlValue::Sequence(vec![YamlValue::Mapping(then_action)]));
        }
    }
    
    Ok(())
}