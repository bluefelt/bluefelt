# Game Implementation Workflow

This guide provides a streamlined workflow for implementing games on Bluefelt. For detailed information on any step, see the linked documentation.

## Quick Overview

Every game implementation follows these 8 steps:

1. **Document Rules** → Create RULES.md
2. **Analyze Requirements** → Map game mechanics to engine capabilities  
3. **Identify Gaps** → Find what the engine can't do yet
4. **Plan Implementation** → Design your approach
5. **Build Game Files** → Create YAML configurations
6. **Add Tests** → Ensure everything works
7. **Validate & Deploy** → Build bundles and run
8. **User Testing** → Get feedback and iterate

## Step 1: Document Game Rules

Create `games/<game-name>/1.0/RULES.md` with:
- Game overview and objectives
- Player count and equipment
- Setup instructions
- Turn structure and actions
- Win conditions
- Special rules and edge cases

📖 [See Rules Documentation Template](./developing-games.md#game-rules)

## Step 2: Analyze System Requirements

Map your game to Bluefelt's architecture:

### What to Analyze:
- **Entities**: What pieces/cards/tokens?
- **Zones**: What boards/areas/hands?  
- **Actions**: What can players do?
- **Phases**: How does game flow?
- **Conditions**: When are actions allowed?

### Check Existing Patterns:
Look at similar games in `games/` directory:
- **Grid games**: tic-tac-toe, connect-four
- **Card games**: Coming soon
- **Movement games**: three-mens-morris

📖 [See State Structure](./state-structure.md) | [Zones Guide](./developing-games-zones.md) | [Entities Guide](./developing-games-entities.md)

## Step 3: Identify Engine Gaps (Critical!)

Before coding, determine what the engine CAN and CANNOT do:

### ✅ Engine CAN Handle:
- Turn-based gameplay
- Grid/list/deck zones  
- Place, move, draw actions
- Line-based win conditions
- Phase transitions
- Automatic triggered actions

### ❌ Engine CANNOT Yet Handle:
- Complex movement patterns (chess-like)
- Area influence calculations
- Real-time mechanics
- Advanced card mechanics

### When You Find a Gap:

1. **Document it clearly** - What exactly is needed?
2. **Propose a generic solution** - How would other games use it?
3. **Work with maintainers** - Get the verb/feature added
4. **Use workarounds** - Or simplify the game design

📖 [See Actions Documentation](./developing-games-actions.md) | [Available Verbs Reference](./developing-games-actions.md#built-in-verbs)

## Step 4: Plan Your Implementation

Based on your analysis, decide your approach:

```
Does the engine support your mechanic?
├─ YES → Use existing verbs/patterns
└─ NO → Can it be made generic?
    ├─ YES → Request new engine verb
    └─ NO → Consider WebAssembly hook
```

### Common Patterns:

**Phase Transitions:**
```yaml
# Automatic action to check conditions and change phase
- id: checkPhaseTransition
  auto: true
  when:
    - condition: zone.count
      with:
        zone: "/zones/board"
        entity: "piece_{player}"
        value: 3
  then:
    - action: setPhase
      with:
        phase: "movement"
```

📖 [See Phases Guide](./developing-games-phases.md) | [See Actions Guide](./developing-games-actions.md)

## Step 5: Build Game Files

Create files in this order:

1. **manifest.yaml** - Game metadata
2. **zones.yaml** - Game boards and areas
3. **entities.yaml** - Pieces and setup
4. **phases.yaml** - Game flow states  
5. **actions.yaml** - Player interactions

### File Structure:
```
games/
  your-game/
    1.0/
      manifest.yaml
      entities.yaml
      zones.yaml
      phases.yaml  
      actions.yaml
      RULES.md
```

### Build and Deploy:
```bash
# Build JSON bundles from YAML
cd cli
cargo run -- build-all

# Server loads from bundles/ directory
cd ../server
cargo run
```

📖 [See Full Implementation Guide](./game-implementation-guide.md)

## Step 6: Add Tests

### Server Tests (`server/tests/your_game_test.rs`):
- Initial state setup
- Each action execution
- Win condition detection
- Edge cases

### Client Tests (`clients/react/src/__tests__/`):
- Component rendering
- Action handling
- State updates

### Always Run Regression Tests:
```bash
# After ANY change, test ALL games
cargo test
cd clients/react && pnpm test
```

📖 [See Testing Guide](./testing-guide.md)

## Step 7: Validate and Deploy

```bash
# Validate game files
./cli/target/debug/bluefelt-cli validate

# Build optimized bundles
./cli/target/debug/bluefelt-cli build-all

# Start server
cd server && cargo run
```

### Manual Testing Checklist:
- [ ] Game loads without errors
- [ ] All actions work correctly
- [ ] Game log shows clear messages
- [ ] Win conditions trigger
- [ ] Mobile experience works
- [ ] No console errors

## Step 8: User Testing

1. Have others play your game
2. Watch for confusion or bugs
3. Note any missing features
4. Iterate based on feedback

## Common Issues and Solutions

### Problem: "My action isn't available"
- Check action conditions
- Verify current phase
- Check zone paths are correct

### Problem: "Log shows ({row}, {col}) placeholders"
- Check what placeholders the verb provides
- Use simpler log templates

### Problem: "Client shows OPERATION_PATH_UNRESOLVABLE"
- Ensure initial state has all needed paths
- Check patch operations target existing paths

📖 [See Troubleshooting Guide](./troubleshooting.md)

## Key Principles

1. **Start Simple** - Get basic gameplay working first
2. **Use Existing Patterns** - Copy from similar games
3. **Test Everything** - Automated and manual testing
4. **Think Generic** - Make features reusable
5. **Document Learnings** - Help the next developer

## Next Steps

- Review [existing games](https://github.com/johnsmith/bluefelt/tree/main/games) for patterns
- Read the [detailed implementation guide](./game-implementation-guide.md) for specific steps
- Check [visual affordances](./visual-affordances.md) for UI patterns
- See [game log parameters](./game-log-parameters.md) for message formatting

Remember: Each game makes the platform better for everyone! 🎮