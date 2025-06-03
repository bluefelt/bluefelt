# Bluefelt CLI

The Bluefelt CLI is a command-line tool for building, validating, and managing games in the Bluefelt ecosystem. Built in Rust, it handles the transformation of human-readable YAML game definitions into optimized JSON bundles that the server can load.

## Overview

The CLI provides essential tools for game developers:
- **Build System** - Convert YAML games to JSON bundles
- **Validation** - Check game files for errors and consistency
- **Scaffolding** - Generate new game templates
- **Watching** - Auto-rebuild during development

## Installation

### From Source

```bash
cd cli
cargo build --release
./target/release/bluefelt-cli --help
```

### Development Build

```bash
cd cli
cargo build
./target/debug/bluefelt-cli --help
```

## Commands

### build

Build a single game from YAML to JSON bundle.

```bash
bluefelt-cli build <GAME_PATH>
```

**Example:**
```bash
# Build tic-tac-toe game
bluefelt-cli build ../games/tic-tac-toe/1.0/

# Output: ../bundles/tic-tac-toe/1.0/*.json
```

**Process:**
1. **Load YAML files** - Read manifest, entities, zones, actions, phases
2. **Expand shorthands** - Process {player} templates and built-ins
3. **Validate structure** - Check required fields and references
4. **Generate JSON** - Convert to optimized JSON format
5. **Write bundles** - Save to bundles directory

### build-all

Build all games in the games directory.

```bash
bluefelt-cli build-all [GAMES_DIR]
```

**Example:**
```bash
# Build all games (default: ../games/)
bluefelt-cli build-all

# Build from custom directory
bluefelt-cli build-all /path/to/games/
```

**Output Structure:**
```
bundles/
├── tic-tac-toe/
│   └── 1.0/
│       ├── manifest.json
│       ├── entities.json
│       ├── zones.json
│       ├── actions.json
│       └── phases.json
├── checkers/
│   └── 1.0/
│       └── ...
└── ...
```

### validate

Validate game files without building.

```bash
bluefelt-cli validate <GAME_PATH>
```

**Example:**
```bash
bluefelt-cli validate ../games/tic-tac-toe/1.0/
```

**Validation Checks:**
- **Required files exist** (manifest.yaml, entities.yaml, zones.yaml, actions.yaml)
- **Valid YAML syntax** in all files
- **Required manifest fields** (gameId, version, specVersion, metadata)
- **Entity/zone ID uniqueness** within each file
- **Action references** point to valid entities/zones
- **Player count consistency** across game definition

### scaffold

Generate a new game template.

```bash
bluefelt-cli scaffold <GAME_NAME> [OUTPUT_DIR]
```

**Example:**
```bash
# Create new game template
bluefelt-cli scaffold my-new-game ../games/

# Creates: ../games/my-new-game/1.0/ with template files
```

**Generated Structure:**
```
my-new-game/
└── 1.0/
    ├── manifest.yaml    # Basic metadata template
    ├── entities.yaml    # Sample entities
    ├── zones.yaml      # Basic zones
    ├── actions.yaml    # Common actions
    └── phases.yaml     # Optional phases
```

### watch

Watch for changes and auto-rebuild during development.

```bash
bluefelt-cli watch <GAME_PATH>
```

**Example:**
```bash
# Watch tic-tac-toe for changes
bluefelt-cli watch ../games/tic-tac-toe/1.0/

# Auto-rebuilds on file changes
```

**Features:**
- **File system monitoring** - Detects changes to .yaml files
- **Incremental builds** - Only rebuilds changed games
- **Error reporting** - Shows validation errors immediately
- **Server integration** - Can optionally restart server

## File Processing

### YAML to JSON Conversion

The CLI handles the transformation from developer-friendly YAML to server-optimized JSON:

**Input (YAML):**
```yaml
# entities.yaml
- id: "piece_{player}"
  type: "piece"
  props:
    color: "{player}_color"
  ui:
    tokenType: "token_{player}"
```

**Output (JSON):**
```json
[
  {
    "id": "piece_p1",
    "type": "piece", 
    "props": {
      "color": "p1_color"
    },
    "ui": {
      "tokenType": "token_p1"
    }
  },
  {
    "id": "piece_p2",
    "type": "piece",
    "props": {
      "color": "p2_color"
    },
    "ui": {
      "tokenType": "token_p2"
    }
  }
]
```

### Shorthand Expansion

The CLI automatically expands several shorthand features:

#### Player Templates

```yaml
# Input
- id: "hand_{player}"
  type: "list"

# Expanded (for 2-player game)
- id: "hand_p1"
  type: "list"
- id: "hand_p2" 
  type: "list"
```

#### Standard Deck

```yaml
# Input
- type: "standardDeck"

# Expanded
- id: "card_hearts_a"
  type: "card"
  props: { suit: "hearts", rank: "A", value: 1 }
- id: "card_hearts_2"
  type: "card" 
  props: { suit: "hearts", rank: "2", value: 2 }
# ... 50 more cards
```

#### Card Actions

```yaml
# Input
- id: "deal_cards"
  uses: "cards.deal"
  with:
    count: 7
    to: "eachPlayer"

# Expanded
- id: "deal_cards"
  auto: true
  then:
    - action: "deal_cards_p1_1"
    - action: "deal_cards_p1_2"
    # ... individual deal actions
```

## Validation System

### File Structure Validation

```rust
pub struct ValidationError {
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
    pub severity: Severity,
}

pub enum Severity {
    Error,   // Prevents building
    Warning, // Builds but may cause issues
    Info,    // Informational notice
}
```

### Common Validation Rules

#### Manifest Validation

```yaml
# Required fields
gameId: "string"           # ✓ Required
version: "string"          # ✓ Required  
specVersion: "string"      # ✓ Required
metadata:                  # ✓ Required
  name: "string"           # ✓ Required
  author: "string"         # ✓ Required
  description: "string"    # ✓ Required
  players:                 # ✓ Required
    min: number            # ✓ Required, > 0
    max: number            # ✓ Required, >= min
```

#### Entity Validation

- **Unique IDs** - No duplicate entity IDs
- **Valid types** - Recognized entity types
- **Consistent player refs** - {player} usage matches player count

#### Zone Validation

- **Unique IDs** - No duplicate zone IDs  
- **Valid types** - grid, list, deck supported
- **Valid contents** - Referenced entities exist
- **Grid dimensions** - rows/cols > 0 for grid zones

#### Action Validation

- **Valid verbs** - Uses known built-in or custom verbs
- **Condition syntax** - Proper condition structure
- **Zone references** - Referenced zones exist

### Error Reporting

```bash
# Example validation output
❌ Error in entities.yaml:15
   Duplicate entity ID: "red_piece"

⚠️  Warning in actions.yaml:23  
   Action "move_piece" has no conditions - may be available when inappropriate

ℹ️  Info: Expanded 2 player templates into 4 entities

❌ Build failed with 1 error, 1 warning
```

## Configuration

### CLI Configuration

The CLI can be configured via command-line flags or environment variables:

```bash
# Command-line flags
bluefelt-cli build --output-dir /custom/bundles ../games/my-game/1.0/
bluefelt-cli validate --strict ../games/my-game/1.0/
bluefelt-cli watch --serve ../games/my-game/1.0/

# Environment variables
BLUEFELT_BUNDLES_DIR=/custom/bundles
BLUEFELT_GAMES_DIR=/custom/games
BLUEFELT_LOG_LEVEL=debug
```

### Build Configuration

Games can include build-specific configuration:

```yaml
# manifest.yaml
build:
  optimize: true           # Minimize JSON output
  validateRefs: strict     # Strict reference checking
  expandAll: true          # Expand all shorthand
  includeDebug: false      # Include debug information
```

## Development Workflow

### Typical Development Cycle

1. **Scaffold** - Create new game template
2. **Edit** - Modify YAML files in your editor
3. **Watch** - Run CLI in watch mode for auto-rebuild
4. **Test** - Start server and test in browser
5. **Validate** - Check for issues before deployment
6. **Build** - Create final bundles for production

### Integration with IDEs

#### VS Code Integration

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build Game",
      "type": "shell",
      "command": "bluefelt-cli",
      "args": ["build", "${workspaceFolder}/games/${input:gameName}/1.0/"],
      "group": "build"
    },
    {
      "label": "Watch Game",
      "type": "shell",
      "command": "bluefelt-cli",
      "args": ["watch", "${workspaceFolder}/games/${input:gameName}/1.0/"],
      "isBackground": true
    }
  ]
}
```

#### YAML Schema Support

The CLI can generate JSON Schema for YAML validation:

```bash
# Generate schema files
bluefelt-cli schema --output schemas/
```

Use in VS Code settings.json:
```json
{
  "yaml.schemas": {
    "./schemas/manifest.schema.json": "**/manifest.yaml",
    "./schemas/entities.schema.json": "**/entities.yaml"
  }
}
```

## Advanced Features

### Custom Validation Rules

Extend validation with custom rules:

```rust
// Custom validator example
pub fn validate_balanced_teams(entities: &[Entity]) -> Vec<ValidationError> {
    let player_entities: HashMap<String, usize> = entities
        .iter()
        .filter(|e| e.props.contains_key("player"))
        .fold(HashMap::new(), |mut acc, e| {
            let player = e.props["player"].as_str().unwrap();
            *acc.entry(player.to_string()).or_insert(0) += 1;
            acc
        });
    
    // Check if all players have equal entities
    // Return ValidationError if imbalanced
}
```

### Bundle Optimization

The CLI can optimize bundles for production:

```bash
# Minified JSON output
bluefelt-cli build --minify ../games/my-game/1.0/

# Remove debug information
bluefelt-cli build --strip-debug ../games/my-game/1.0/

# Compress with gzip
bluefelt-cli build --compress ../games/my-game/1.0/
```

### Dependency Management

Handle dependencies between games:

```yaml
# manifest.yaml
dependencies:
  - gameId: "card-game-base"
    version: "^1.0"
    components: ["standardDeck", "cardActions"]
```

### Localization Support

Support multiple languages:

```yaml
# manifest.yaml
localization:
  defaultLocale: "en"
  supportedLocales: ["en", "es", "fr"]

# entities.yaml
- id: "red_piece"
  ui:
    name:
      en: "Red Piece"
      es: "Pieza Roja"
      fr: "Pièce Rouge"
```

## Troubleshooting

### Common Build Issues

**YAML Syntax Errors:**
```bash
❌ Error in entities.yaml:15:3
   Invalid YAML syntax: expected sequence entry

# Fix: Check indentation and YAML structure
```

**Missing Required Fields:**
```bash
❌ Error in manifest.yaml
   Missing required field: metadata.players.max

# Fix: Add all required manifest fields
```

**Reference Errors:**
```bash
❌ Error in actions.yaml:23
   Action references unknown entity: "invalid_piece"

# Fix: Ensure all entity/zone references are valid
```

**Player Count Mismatch:**
```bash
⚠️  Warning: Entity "piece_{player}" expanded for 2 players, 
   but zone "supply_{player}" references 4 players

# Fix: Consistent player count across all files
```

### Debug Mode

Enable verbose output for troubleshooting:

```bash
# Detailed logging
RUST_LOG=debug bluefelt-cli build ../games/my-game/1.0/

# Show expansion process
bluefelt-cli build --verbose ../games/my-game/1.0/

# Dry run (validate without writing)
bluefelt-cli build --dry-run ../games/my-game/1.0/
```

### Performance Issues

**Large Games:**
- Use incremental builds with watch mode
- Enable build caching for dependencies
- Consider splitting into multiple smaller games

**Complex Expansions:**
- Profile shorthand expansion performance
- Optimize {player} template usage
- Cache expanded components

## Testing Integration

### CI/CD Integration

```yaml
# GitHub Actions example
name: Build Games
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build CLI
        run: cd cli && cargo build --release
      - name: Validate All Games
        run: ./cli/target/release/bluefelt-cli validate-all games/
      - name: Build All Games  
        run: ./cli/target/release/bluefelt-cli build-all games/
      - name: Upload Bundles
        uses: actions/upload-artifact@v3
        with:
          name: game-bundles
          path: bundles/
```

### Testing Built Games

```bash
# Build and test game
bluefelt-cli build ../games/my-game/1.0/
cd ../server && cargo run &
cd ../clients/react && pnpm dev

# Automated testing  
bluefelt-cli test ../games/my-game/1.0/ --scenarios test-scenarios.yaml
```

The Bluefelt CLI provides comprehensive tooling for the entire game development lifecycle, from initial scaffolding through production deployment, with robust validation and optimization features.