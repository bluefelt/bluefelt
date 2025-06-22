# View Zones

View zones are special zones that display strategic information rather than containing entities. They provide a way to show computed game state, statistics, and summaries in a generic way that clients can render without game-specific logic.

## Implementation Status

View zones are now fully implemented with support for:
- Expression evaluation: `count()`, `sum()`, `avg()`, `max()`, `min()`
- State path references: `state.scores.{player}`
- Mathematical expressions: `count(pairs_{player}) / 2`
- Per-player and shared data fields
- Multiple view types: strategic, log, progress, summary

## Concept

Traditional zones contain entities (cards, tokens, etc.) that can be moved and manipulated. View zones contain structured data that reflects the current game state, providing players with strategic information at a glance.

## Zone Definition

```yaml
- id: game_overview
  name: "Game Overview"
  shape: view
  visibility: public
  viewType: strategic
  viewData:
    - field: pairsMade
      label: "Pairs Made"
      source: "count(pairs_{player})"
      perPlayer: true
```

## View Types

### Strategic View
Shows key game metrics and statistics:
- Score tracking
- Resource counts
- Achievement progress
- Player comparisons

### Log View
Shows recent game activity:
- Action history
- Important events
- Turn summaries

### Progress View
Shows game progression:
- Phase tracking
- Round information
- Time remaining

## Data Sources

View zones can pull data from:
- Zone counts: `count(zone_id)`
- Entity properties: `sum(zone_id.value)`
- Game state: `state.currentRound`
- Computed values: `max(scores)`, `avg(hand_sizes)`

## Client Rendering

Clients render view zones generically based on the data structure:

```json
{
  "id": "game_overview",
  "shape": "view",
  "viewType": "strategic",
  "data": {
    "p1": {
      "pairsMade": 3,
      "cardsInHand": 5
    },
    "p2": {
      "pairsMade": 2,
      "cardsInHand": 7
    },
    "shared": {
      "poolRemaining": 20
    }
  }
}
```

## Benefits

1. **No Game-Specific UI**: Clients render view zones generically
2. **Flexible Display**: Games can show whatever information is strategic
3. **Consistent Updates**: View data updates automatically with game state
4. **Accessibility**: Important information is structured and labeled

## Examples

### Go Fish Overview
```yaml
- id: game_overview
  shape: view
  viewData:
    - field: pairsMade
      source: "count(pairs_{player}) / 2"
      perPlayer: true
    - field: cardsInHand
      source: "count(hand_{player})"
      perPlayer: true
```

### Trick-Taking Round Summary
```yaml
- id: round_summary
  shape: view
  viewData:
    - field: tricksWon
      source: "state.tricksWon.{player}"
      perPlayer: true
    - field: currentBid
      source: "state.bids.{player}"
      perPlayer: true
```

### Resource Game Status
```yaml
- id: resource_display
  shape: view
  viewData:
    - field: gold
      source: "sum(treasury_{player}.value)"
      perPlayer: true
    - field: buildings
      source: "count(buildings_{player})"
      perPlayer: true
```

## Implementation Notes

View zones are computed server-side and sent as structured data. They don't participate in action targeting or entity movement. The server updates view zone data whenever the underlying game state changes.

This approach keeps the client generic while allowing games to display rich strategic information in a consistent, accessible way.