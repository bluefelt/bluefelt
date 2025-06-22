#[cfg(test)]
mod tests {
    use super::*;
    use crate::yaml_shortcuts::expand_shortcuts;
    use crate::yaml_includes::process_includes;
    use serde_yaml::Value as YamlValue;
    use std::collections::HashSet;
    use std::path::Path;

    #[test]
    fn test_if_then_else_shorthand() {
        let yaml_str = r#"
        actions:
          - id: test_action
            if: "zone.isEmpty"
            uses: "someAction"
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        // Check that 'if' was converted to 'when'
        let actions = &expanded["actions"].as_sequence().unwrap()[0];
        assert!(actions.get("when").is_some());
        assert!(actions.get("if").is_none());
        
        // Check the condition structure
        let when_conditions = actions["when"].as_sequence().unwrap();
        assert_eq!(when_conditions.len(), 1);
        assert_eq!(when_conditions[0]["condition"].as_str().unwrap(), "zone.isEmpty");
    }

    #[test]
    fn test_bf_standard_library_actions() {
        let yaml_str = r#"
        actions:
          - id: deal_cards
            uses: "bf.deal"
          - id: draw_card
            uses: "bf.draw"
          - id: shuffle_deck
            uses: "bf.shuffle"
          - id: next_turn
            uses: "bf.nextTurn"
          - id: end_game
            uses: "bf.endGame"
          - id: place_token
            uses: "bf.place"
          - id: move_piece
            uses: "bf.move"
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        let actions = expanded["actions"].as_sequence().unwrap();
        
        // Check that bf.* actions were expanded to proper verbs
        assert_eq!(actions[0]["uses"].as_str().unwrap(), "dealEntities");
        assert_eq!(actions[1]["uses"].as_str().unwrap(), "transferEntity");
        assert_eq!(actions[2]["uses"].as_str().unwrap(), "shuffle");
        assert_eq!(actions[3]["uses"].as_str().unwrap(), "nextTurn");
        assert_eq!(actions[4]["uses"].as_str().unwrap(), "endGame");
        assert_eq!(actions[5]["uses"].as_str().unwrap(), "placeEntity");
        assert_eq!(actions[6]["uses"].as_str().unwrap(), "moveEntity");
        
        // Check that bf.deal has default parameters
        assert!(actions[0]["with"].is_mapping());
        let deal_params = &actions[0]["with"];
        assert_eq!(deal_params["from"].as_str().unwrap(), "deck");
        assert_eq!(deal_params["to"].as_str().unwrap(), "hand_{player}");
        assert_eq!(deal_params["count"].as_i64().unwrap(), 7);
    }

    #[test]
    fn test_condition_object_shortcuts() {
        let yaml_str = r#"
        actions:
          - id: test1
            if:
              phase: "main"
            uses: "someAction"
          - id: test2
            if:
              owner: "{player}"
            uses: "someAction"
          - id: test3
            if:
              empty: true
            uses: "someAction"
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        let actions = expanded["actions"].as_sequence().unwrap();
        
        // Check phase condition
        let phase_condition = &actions[0]["when"].as_sequence().unwrap()[0];
        assert_eq!(phase_condition["condition"].as_str().unwrap(), "phase.is");
        assert_eq!(phase_condition["with"]["phase"].as_str().unwrap(), "main");
        
        // Check owner condition
        let owner_condition = &actions[1]["when"].as_sequence().unwrap()[0];
        assert_eq!(owner_condition["condition"].as_str().unwrap(), "entity.owner");
        assert_eq!(owner_condition["with"]["owner"].as_str().unwrap(), "{player}");
        
        // Check empty condition
        let empty_condition = &actions[2]["when"].as_sequence().unwrap()[0];
        assert_eq!(empty_condition["condition"].as_str().unwrap(), "zone.isEmpty");
    }

    #[test]
    fn test_standard_deck_shortcut() {
        let yaml_str = r#"
        deck: standard-52
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        // Check that standard-52 was expanded to 52 cards
        let entities = expanded["entities"].as_sequence().unwrap();
        assert_eq!(entities.len(), 52);
        
        // Check a few cards
        let first_card = &entities[0];
        assert_eq!(first_card["type"].as_str().unwrap(), "card");
        assert!(first_card["id"].as_str().unwrap().starts_with("card_"));
        assert!(first_card["properties"]["rank"].is_string());
        assert!(first_card["properties"]["suit"].is_string());
    }

    #[test]
    fn test_grid_shortcut() {
        let yaml_str = r#"
        zones:
          - id: game_board
            grid: 3x3
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        let zone = &expanded["zones"].as_sequence().unwrap()[0];
        assert_eq!(zone["type"].as_str().unwrap(), "board");
        assert_eq!(zone["rows"].as_i64().unwrap(), 3);
        assert_eq!(zone["cols"].as_i64().unwrap(), 3);
    }

    #[test]
    fn test_log_shortcut() {
        let yaml_str = r#"
        actions:
          - id: test_action
            uses: "someAction"
            log: "Player {player} did something"
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        let action = &expanded["actions"].as_sequence().unwrap()[0];
        assert!(action.get("log").is_none());
        assert_eq!(
            action["ui"]["logTemplate"].as_str().unwrap(),
            "Player {player} did something"
        );
    }

    #[test]
    fn test_then_string_shortcut() {
        let yaml_str = r#"
        actions:
          - id: test_action
            uses: "someAction"
            then: "nextTurn"
        "#;
        
        let yaml: YamlValue = serde_yaml::from_str(yaml_str).unwrap();
        let expanded = expand_shortcuts(yaml).unwrap();
        
        let action = &expanded["actions"].as_sequence().unwrap()[0];
        let then_actions = action["then"].as_sequence().unwrap();
        assert_eq!(then_actions.len(), 1);
        assert_eq!(then_actions[0]["uses"].as_str().unwrap(), "nextTurn");
    }
}