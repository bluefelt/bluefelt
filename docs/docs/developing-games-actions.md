# Developing Games: Actions

Actions define what players can do in your game - from placing pieces to drawing cards. This guide covers how to create actions using Bluefelt's built-in verbs and action system.

## Overview

Actions in Bluefelt handle:
- **Basic Moves** - Place pieces, move entities, draw cards
- **Game Flow** - End turn, change phase
- **Automated Actions** - Triggered by game events
- **Chained Actions** - Actions that trigger other actions

Each action specifies what it does and optionally when it's available.

## Action Structure

### Minimal Action

```yaml
- id: "place_piece"
  uses: "place"
```

### Complete Action Structure

```yaml
- id: "advanced_move"
  uses: "moveEntity"
  ui:
    name: "Tactical Movement"
    description: "Move a unit and potentially attack"
    direction: "Select a unit to move"
    icon: "🏃"
    color: "#0066cc"
    hotkey: "M"
    priority: 1
  conditions:
    - type: "current_player_turn"
    - type: "zone_contains_player_entity"
      zone: "{source}"
    - type: "path_clear"
      from: "{source}"
      to: "{target}"
    - type: "movement_range"
      entity: "{source_entity}"
      range: 3
  effects:
    - verb: "moveEntity"
      args:
        from: "{source}"
        to: "{target}"
    - trigger: "check_combat"
      if: "enemy_adjacent"
  triggers:
    - "update_visibility"
    - "check_victory_conditions"
  auto: false
  availability: "always"
  cooldown: 0
  cost:
    - resource: "action_points"
      amount: 1
  metadata:
    category: "movement"
    complexity: "medium"
    tutorial: "Move units to strategic positions"
```
