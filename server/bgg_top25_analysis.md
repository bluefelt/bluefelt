# BoardGameGeek Top 25 Analysis: Common Eurogame Patterns

## Games Analyzed (Rankings 10-25)

### 10. **Spirit Island** (2017)
- **Core Mechanics**: Cooperative, Hand Management, Simultaneous Action Selection, Variable Player Powers
- **Key Systems**: Spirit growth/presence spreading, power card acquisition, fear generation, invader phase automation
- **Unique Features**: Reverse colonization theme, modular difficulty, complex multi-use cards

### 11. **Gloomhaven: Jaws of the Lion** (2020)
- **Core Mechanics**: Card-Based Combat, Hand Management, Campaign/Legacy, Grid Movement
- **Key Systems**: Initiative system, attack modifiers, character progression, scenario-based play
- **Unique Features**: Tutorial campaign, streamlined setup via scenario book, persistent character development

### 12. **Gaia Project** (2017)
- **Core Mechanics**: Variable Player Powers, Tech Trees, Area Control, Resource Management
- **Key Systems**: Power cycling, terraforming, building upgrades, research tracks
- **Unique Features**: 14 asymmetric factions, space theme adaptation of Terra Mystica

### 13. **Twilight Struggle** (2005)
- **Core Mechanics**: Card-Driven, Area Control, Hand Management, Tug-of-War
- **Key Systems**: Operations vs events decisions, DEFCON track, space race, scoring cards
- **Unique Features**: Historical event integration, multiple victory conditions, tension without elimination

### 14. **Through the Ages: A New Story of Civilization** (2015)
- **Core Mechanics**: Card Drafting, Civilization Building, Resource Management, Engine Building
- **Key Systems**: Population management, technology development, military conflicts, culture generation
- **Unique Features**: No map board, pure card-based civ building, interconnected resource systems

### 15. **The Castles of Burgundy** (2011)
- **Core Mechanics**: Dice Placement, Tile Placement, Set Collection, Pattern Building
- **Key Systems**: Dice mitigation via workers, region completion bonuses, goods trading
- **Unique Features**: Dice-driven euro, chain reaction combos, polyomino-like placement

### 16. **Dune: Imperium – Uprising** (2023)
- **Core Mechanics**: Deck Building + Worker Placement Hybrid, Hand Management
- **Key Systems**: Card-driven worker placement, combat resolution, faction influence, spies
- **Unique Features**: Seamless integration of two major mechanics, sandworm battles, spy infiltration

### 17. **Great Western Trail** (2016)
- Already analyzed in previous request

### 18. **Scythe** (2016)
- **Core Mechanics**: Engine Building, Area Control, Variable Player Powers, Action Selection
- **Key Systems**: Top/bottom action matrix, resource production, mech deployment, popularity track
- **Unique Features**: Euro/4X hybrid, no player elimination, asymmetric factions and player boards

### 19. **Eclipse: Second Dawn for the Galaxy** (2020)
- **Core Mechanics**: 4X (Explore, Expand, Exploit, Exterminate), Tech Trees, Dice Combat
- **Key Systems**: Action economy with upkeep costs, ship customization, exploration tiles
- **Unique Features**: Streamlined 4X in 2-3 hours, modular ship building, procedural galaxy

### 20. **7 Wonders Duel** (2015)
- **Core Mechanics**: Card Drafting, Set Collection, Resource Management
- **Key Systems**: Pyramid card display, three victory conditions, wonder construction
- **Unique Features**: Two-player specific design, tableau drafting instead of hand passing

### 21. **Brass: Lancashire** (2007)
- **Core Mechanics**: Economic Engine Building, Network Building, Hand Management
- **Key Systems**: Canal/rail eras, industry development, coal/iron markets, beer production
- **Unique Features**: Shared resource economy, overbuilding mechanics, two distinct game phases

### 22. **Nemesis** (2018)
- **Core Mechanics**: Semi-Cooperative, Hand Management, Dice Combat, Hidden Objectives
- **Key Systems**: Noise generation, contamination/injuries, escape conditions, alien AI
- **Unique Features**: Survival horror theme, betrayal possibilities, cinematic moments

### 23. **Clank! Legacy: Acquisitions Incorporated** (2019)
- **Core Mechanics**: Deck Building, Press Your Luck, Legacy/Campaign, Dungeon Crawl
- **Key Systems**: Clank generation, artifact collection, dragon attacks, permanent changes
- **Unique Features**: Legacy deck building, humor/narrative focus, evolving board

### 24. **A Feast for Odin** (2016)
- **Core Mechanics**: Worker Placement, Polyomino Tile Placement, Resource Management
- **Key Systems**: 61 action spaces, income generation, island colonization, feast requirements
- **Unique Features**: Massive action selection, tetris-like placement puzzles, Viking theme integration

### 25. **Concordia** (2013)
- **Core Mechanics**: Hand Management, Deck Building, Network Building, Resource Management
- **Key Systems**: Role selection via cards, goods production/trade, scoring via god cards
- **Unique Features**: No dice/randomness, multi-use cards, elegant simplicity

## Common Patterns Requiring SDK Support

### 1. **Multi-Use Cards/Resources**
- **Found in**: Spirit Island, Gloomhaven, Twilight Struggle, Dune Imperium, Concordia
- **Pattern**: Cards/resources that can be used in multiple ways, forcing tactical decisions
- **SDK Need**: Flexible action system supporting conditional card plays and multiple usage modes

### 2. **Action Selection Systems**
- **Found in**: Almost all games analyzed
- **Variants**: 
  - Worker placement (Feast for Odin, Dune Imperium)
  - Card-driven (Twilight Struggle, Concordia)
  - Action matrix (Scythe)
  - Initiative-based (Gloomhaven)
- **SDK Need**: Modular action system with constraints, costs, and availability tracking

### 3. **Resource Conversion Chains**
- **Found in**: Gaia Project, Through the Ages, Brass, Feast for Odin
- **Pattern**: Complex resource transformation systems with multiple steps
- **SDK Need**: Resource pipeline management, conversion rules, and efficiency tracking

### 4. **Area Control/Influence**
- **Found in**: Twilight Struggle, Gaia Project, Scythe, Eclipse, Brass
- **Pattern**: Territorial control with various benefits and contested spaces
- **SDK Need**: Zone ownership, influence tracking, adjacency rules, and control benefits

### 5. **Tech Trees/Progression Tracks**
- **Found in**: Gaia Project, Through the Ages, Eclipse, Spirit Island
- **Pattern**: Branching advancement paths with prerequisites and bonuses
- **SDK Need**: Progression system with dependencies, unlocks, and persistent upgrades

### 6. **Variable Player Powers**
- **Found in**: Spirit Island, Gaia Project, Scythe, Eclipse, Gloomhaven
- **Pattern**: Asymmetric starting conditions and unique abilities per player/faction
- **SDK Need**: Faction system with unique rules, starting conditions, and special abilities

### 7. **Hidden Information/Objectives**
- **Found in**: Nemesis, Twilight Struggle, Scythe
- **Pattern**: Secret goals that affect player behavior and create tension
- **SDK Need**: Private information management, reveal timing, and objective tracking

### 8. **Tile/Card Placement Puzzles**
- **Found in**: Castles of Burgundy, Feast for Odin, 7 Wonders Duel
- **Pattern**: Spatial puzzles where placement order and position matter
- **SDK Need**: Grid/tableau system with placement rules, adjacency bonuses, and constraints

### 9. **Economic Engines**
- **Found in**: Brass, Through the Ages, Scythe, Concordia
- **Pattern**: Building production chains that generate resources/points over time
- **SDK Need**: Production cycles, supply/demand mechanics, and market systems

### 10. **Phased Gameplay**
- **Found in**: Brass (Canal/Rail), Through the Ages (Ages), Eclipse (Rounds)
- **Pattern**: Distinct game phases with different rules or opportunities
- **SDK Need**: Phase management with rule modifications and transitions

### 11. **Deck Building Integration**
- **Found in**: Dune Imperium, Clank!, Concordia
- **Pattern**: Personal deck construction affecting available actions
- **SDK Need**: Deck manipulation, card acquisition, and deck cycling mechanics

### 12. **Combat Resolution Systems**
- **Found in**: Nemesis, Eclipse, Gloomhaven, Twilight Struggle
- **Variants**: Dice-based, card-driven, deterministic
- **SDK Need**: Flexible combat framework supporting different resolution methods

## Key SDK Design Considerations

1. **Modularity**: Systems should be mix-and-match compatible
2. **Scalability**: Support games from 2-player to 5+ players
3. **Information Scoping**: Handle public, private, and hidden information
4. **State Persistence**: Support for campaign/legacy elements
5. **AI Automation**: Enemy/neutral force automation (invaders, aliens, etc.)
6. **Timing Windows**: Interrupt actions, responses, and simultaneous play
7. **Resource Abstraction**: Flexible enough to handle various resource types
8. **Victory Conditions**: Support for multiple, simultaneous win conditions
9. **Component Limits**: Track limited supplies of pieces/cards
10. **Undo/Redo Support**: For complex games with many steps per turn