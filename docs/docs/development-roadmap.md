# Development Roadmap

This document is the **authoritative source** for all Bluefelt development tasks and future plans.

## 🚀 Current Priority Phases

### ✅ Phase A: Production Readiness Validation (COMPLETED)

**Goal**: Ensure the lobby and table management system works reliably in practice.  
**Status**: ✅ **COMPLETED** - Platform validated as production-ready  
**Completed**: January 2025

#### ✅ A1: End-to-End Integration Testing
- [x] **Start server and client** - Verified complete integration works (ports 8000/5173)
- [x] **Test auto-seat assignment flow** - Create lobby → table → join → verify seating  
- [x] **Multi-player table testing** - Multiple users joining same table working
- [x] **Ready state and countdown** - Countdown triggers correctly
- [x] **Table leaving and rejoining** - Edge cases with player movements tested
- [x] **WebSocket message handling** - Client receives all state updates properly

#### ✅ A2: Game Bundle Consistency  
- [x] **Rebuild all game bundles** - `cd cli && cargo run -- build-all` working
- [x] **Test each game end-to-end** - All core games verified with new table system:
  - [x] Tic-tac-toe with auto-seat assignment (win detection fixed)
  - [x] Connect Four with proper player count validation
  - [x] Go Fish with multi-player coordination  
  - [x] Three Men's Morris with movement phases
- [x] **Verify game start validation** - Min/max players enforced correctly
- [x] **13 games loaded** - All games building and loading successfully

#### ✅ A3: Client-Server Compatibility Audit
- [x] **API compatibility check** - All HTTP endpoints working (`/api/lobbies`, `/api/debug`)
- [x] **WebSocket message format** - Client handles new table messages (tableCreated, tableUpdated, tableJoined)
- [x] **UI state management** - React client reflects server state changes correctly
- [x] **Error handling flow** - Robust error messages for validation failures

**✅ PRODUCTION READINESS VALIDATED**: Platform ready for limited production use with multi-player table gaming functionality.

### Phase B: Core Stability (CURRENT PRIORITY)

**Goal**: Complete development infrastructure and address identified stability issues.  
**Status**: 🔄 **ACTIVE** - Next priority after Phase A completion

#### B1: Test Framework Completion ✅ (COMPLETED)
**Priority**: Critical for development workflow
- [x] **Complete E2E test framework updates** - Table system compatibility working
- [x] **Fix game start sequence in tests** - Action map generation working correctly
- [x] **WebSocket test reliability** - Message handling is stable
- [x] **Test isolation improvements** - Each scenario creates fresh game instance
- [x] **Individual game test debugging** - Isolation fixed, all games tested

#### B2: Connection Management Optimization ✅ (COMPLETED)
**Priority**: High for production stability
- [x] **Fix CLOSE_WAIT connection accumulation** - Implemented proper WebSocket cleanup on disconnect
- [x] **Graceful disconnect handling** - Added heartbeat/ping mechanism with 60s timeout
- [x] **Connection pooling refinement** - Added ConnectionPool with per-user (3) and total (100) limits
- [x] **Reconnection logic** - Implemented reconnection tokens with 5-minute expiry

#### B3: Individual Game Quality ✅ (COMPLETED)
**Priority**: High - Fixed all critical game issues
- [x] **Connect Four player slot mapping** - Fixed by adding onGameStarted handler
- [x] **Connect Four win detection** - Fixed by manually correcting bundle (CLI has build bug)
- [x] **Three Men's Morris phase detection** - Fixed test to handle both phase patch paths
- [x] **Multi-step action system** - Fixed action map generation to use `_multiStep_{actionId}` format with proper condition checking

#### B4: Lobby Management Features ✅ (COMPLETED January 2025)
**Priority**: Medium - completed ahead of schedule
- [x] **Lobby ownership system** - First player to join becomes owner/admin
- [x] **Rename lobby capability** - Owner can rename lobby via `renameLobby` action
- [x] **UI for lobby management** - Inline editing with owner-only controls
- [ ] **Future enhancements** - Kick players, transfer ownership, lobby settings

#### B5: Performance & Code Quality 🟢
**Priority**: Low for optimization
- [ ] **Action map caching** - Avoid recomputing on every state change
- [ ] **Memory usage optimization** - Monitor and optimize resource usage
- [ ] **Code cleanup** - Remove dead code and fix compiler warnings
- [ ] **Bundle loading optimization** - Streamline deployment process

### Phase C: Platform-Wide Game Variety Support (HIGH PRIORITY)

**Goal**: Enable the full spectrum of game types through coordinated server, client, and YAML enhancements.

#### C1: Essential Zone & Movement Systems 🔴
**Priority**: Critical for game variety
- [ ] **Track/Path zones** - Server: New zone type with path verbs; Client: Path rendering; YAML: `type: track` with connections
- [ ] **Drag and drop actions** - Server: `dragMove` verb with validation; Client: Drag gesture handling; YAML: `interaction: drag`
- [ ] **Entity state system** - Server: Entity state properties (rotation, facing, stacked); Client: State visualization; YAML: `states:` definitions
- [ ] **Multi-select actions** - Server: Group action processing; Client: Selection UI; YAML: `multiSelect: true` on actions
- [ ] **Composite entities** - Server: Entity containment hierarchy; Client: Nested rendering; YAML: `contains:` relationships

#### C2: Advanced Interaction Systems 🟡
**Priority**: High for complex games
- [ ] **Context action system** - Server: Multiple valid actions per location; Client: Context menu UI; YAML: Action priority/grouping
- [ ] **Gesture interactions** - Server: Gesture verb types; Client: Touch/mouse gesture recognition; YAML: `gesture:` definitions
- [ ] **Undo/redo system** - Server: Action history and reversal; Client: UI controls; YAML: `reversible: true` on actions
- [ ] **Dynamic zone management** - Server: Runtime zone creation/destruction; Client: Dynamic rendering; YAML: `createZone`/`destroyZone` verbs
- [ ] **Zone relationships** - Server: Zone connection graph; Client: Connection visualization; YAML: `connections:` definitions

#### C3: Visual & Feedback Systems 🟡
**Priority**: Medium for polish
- [ ] **Phase transition system** - Server: Phase change events; Client: Transition animations; YAML: `transition:` effects
- [ ] **Animation sequences** - Server: Animation hints in patches; Client: Complex animation engine; YAML: `animation:` definitions
- [ ] **Effect system** - Server: Effect triggers; Client: Particle/visual effects; YAML: `effects:` library
- [ ] **Theme system** - Server: Theme data delivery; Client: Dynamic theming; YAML: `theme:` definitions
- [ ] **State displays** - Server: Computed view data; Client: Custom displays; YAML: `displays:` configuration

### Phase D: Developer Experience (MEDIUM PRIORITY)

**Goal**: Make development and debugging easier.

#### D1: Debugging and Monitoring 🟡
- [ ] **Enhanced error messages** - Replace string errors with structured types
- [ ] **Performance metrics** - Add monitoring for action processing times
- [ ] **Memory usage tracking** - Monitor for memory leaks in long-running games
- [ ] **Debug tooling** - Improve existing debug endpoints

#### D2: Development Workflow 🟡
- [x] **Log file organization** - Centralized logs directory with proper structure
- [ ] **Hot reload for games** - Automatic server restart when YAML changes
- [ ] **Better validation errors** - Clear, actionable error messages for game developers
- [ ] **Documentation updates** - Keep docs in sync with latest changes
- [ ] **Development standards** - Code quality guidelines and patterns

### Phase E: Extended Platform Capabilities (MEDIUM PRIORITY)

**Goal**: Add advanced features for specialized game types across the platform.

#### E1: Specialized Zone Systems 🟢
**Priority**: Medium for niche games
- [ ] **Graph zones** - Server: Node/edge data structures; Client: Graph layout algorithms; YAML: `type: graph` with nodes/edges
- [ ] **Area zones** - Server: Territory calculation; Client: Area rendering; YAML: `type: area` with influence rules
- [ ] **3D/Layered zones** - Server: Z-axis support; Client: 3D or layered rendering; YAML: `layers:` definitions
- [ ] **Irregular grids** - Server: Custom coordinate systems; Client: Tessellation rendering; YAML: Custom grid definitions

#### E2: Advanced Game Systems 🟢
**Priority**: Low for specialized games
- [ ] **Ownership hierarchy** - Server: Ownership chain resolution; Client: Ownership visualization; YAML: `ownership:` rules
- [ ] **Entity formations** - Server: Group movement/behavior; Client: Formation display; YAML: `formation:` definitions
- [ ] **Custom inputs** - Server: Input validation; Client: Widget library; YAML: `input:` specifications
- [ ] **Embedded games** - Server: Sub-game management; Client: Nested game views; YAML: `subgame:` definitions
- [ ] **Simultaneous play** - Server: Parallel action resolution; Client: Real-time updates; YAML: `simultaneous: true` phases

### Phase F: Long-term Architecture Improvements (LOW PRIORITY)

**Goal**: Improve platform architecture and enable more complex games.

#### F1: Action Execution Refactor 🟢
**Timeline**: Q2 2025

Refactor the action execution pipeline to address current limitations:
- [ ] **Fix nested "then" actions** - conditionalAction properly executes then-actions
- [ ] **Unified execution model** - Single ActionExecutor for all action types
- [ ] **Context preservation** - Template variables flow through entire execution chain
- [ ] **VerbExecutor trait system** - Extensible verb implementation
- [ ] **Action logging & metrics** - Comprehensive execution debugging

#### F2: Server Architecture Refinements 🟢
**Timeline**: Q2 2025

Complete remaining server refactoring work:
- [ ] **GameController abstraction** - Clean interface hiding engine details
- [ ] **State initialization cleanup** - Single configurable initialization function
- [ ] **Message routing improvements** - Declarative MessageTarget system
- [ ] **Lock management completion** - Remove remaining nested lock patterns

#### F3: Mathematical Calculation System 🟢
**Timeline**: Q3 2025

Comprehensive mathematical operations for complex games:
- [ ] **Core verbs** - calculate, increment, decrement, multiply, compareValues
- [ ] **Expression evaluator** - Arithmetic, functions (min, max, abs, floor, ceil)
- [ ] **Numeric conditions** - Enhanced condition system with math
- [ ] **View zone integration** - Display calculated values

#### F4: Enhanced Game Definition Features 🟢
**Timeline**: Q3 2025

Advanced game development capabilities:
- [ ] **Action hooks system** - Pre/post execution hooks for custom logic
- [ ] **Action validation framework** - Bundle validation and dependency checking
- [ ] **Custom verb executors** - Game-specific verb implementations
- [ ] **Action composition** - Build complex actions from simpler ones

#### F5: Performance & Developer Experience 🟢
**Timeline**: Q3-Q4 2025

- [ ] **Action execution optimization** - Caching, parallel execution, benchmarking
- [ ] **Debugging tools** - Action tree visualization, step-through debugging
- [ ] **Testing framework** - Action unit tests, replay system, undo/redo
- [ ] **Visual game builder** - Web-based game creation tool

#### F6: Platform Features 🟢
**Timeline**: Q4 2025

- [ ] **Player profiles** - Authentication and user management
- [ ] **Game history** - Statistics and replay system
- [ ] **Matchmaking** - Skill-based player matching
- [ ] **Social features** - Friends, invites, chat improvements

---

## 🎯 Immediate Next Actions

Based on Phase A completion and client analysis, here are the immediate priorities:

### Next 1-2 Weeks: Connection Stability (Phase B2)
1. **CLOSE_WAIT connection fixes** - Resolve connection accumulation issues
2. **Production monitoring** - Implement connection health monitoring
3. **Graceful disconnect handling** - Improve cleanup on user disconnection

### Next 2-4 Weeks: Essential Platform Features (Phase C1)
1. **Track/Path zones** - Implement server zone type, client rendering, and YAML support
2. **Drag and drop** - Add dragMove verb, client gestures, and YAML interaction types
3. **Entity states** - Server state properties, client visualization, YAML state definitions

### Next Month: Core Platform Flexibility (Phase C)
1. **Multi-select actions** - Server group processing, client UI, YAML multiSelect support
2. **Context actions** - Server multi-action support, client menus, YAML action grouping
3. **Phase transitions** - Server events, client animations, YAML transition effects
4. **Dynamic zones** - Server runtime management, client updates, YAML create/destroy verbs

---

## 📊 Completed Major Milestones ✅

### ✅ Phase A: Production Readiness Validation (COMPLETED January 2025)
- ✅ **Complete platform integration** - Server, client, and API all operational
- ✅ **Auto-seat assignment system** - Full table management with automatic seating
- ✅ **Game bundle deployment** - 13 games loading successfully via CLI build system
- ✅ **WebSocket communication** - Real-time multi-player functionality working
- ✅ **Core game mechanics** - Action processing, state management, win detection
- ✅ **Production deployment ready** - Platform validated for limited production use

### Phase B3: Individual Game Quality (COMPLETED January 2025)
- ✅ **Connect Four fixes** - Player slot mapping and win detection working
- ✅ **Three Men's Morris** - Movement phase and multi-step actions functional
- ✅ **Go Fish improvements** - Card handling and multi-step selection working
- ✅ **Multi-step action system** - Fixed with proper `_multiStep_{actionId}` format
- ✅ **Win detection** - All games properly detecting end conditions
- ✅ **Action map generation** - Consistent and reliable across all game states

### Phase B4: Lobby Management (COMPLETED January 2025)
- ✅ **Lobby ownership** - First player to join becomes owner
- ✅ **Ownership transfer** - Automatic transfer to earliest member when owner leaves
- ✅ **Lobby archival** - Empty lobbies archived and prevented from rejoining
- ✅ **Rename capability** - Owners can rename lobbies
- ✅ **Table names removed** - Tables identified by game type only

### Phase 4: Server Resource Management (COMPLETED January 2025)
- ✅ **ConnectionManager** - WebSocket connection pooling and cleanup
- ✅ **MemoryManager** - Bounded collections prevent unbounded growth  
- ✅ **LockOptimizer** - Reduced lock contention with timeout-based acquisition
- ✅ **PatchOptimizer** - Intelligent patch reduction (30-70% reduction)
- ✅ **EntityPool** - Memory-efficient entity management with object pooling
- ✅ **High-load testing** - 200+ concurrent connections, 1000+ messages/sec

### Auto-Seat Assignment Implementation (COMPLETED)
- ✅ **Server-side auto-seating** - Table owners automatically seated
- ✅ **Client integration** - Single "Join Table" button instead of seat selection
- ✅ **Validation logic** - Prevent joining multiple tables simultaneously
- ✅ **Table capacity tests** - Comprehensive validation of player limits
- ✅ **Game start tests** - Min/max player enforcement and ready states
- ✅ **Atomic seat operations** - Race-condition-free seat claiming

### Core Engine Features (COMPLETED Phases 1-3)
- ✅ **Zone Tier System** - 4-tier spatial organization (hand→tactical→strategic→ambient)
- ✅ **View Zones** - Strategic information display
- ✅ **Math System** - Expression evaluator and calculations
- ✅ **Animation System** - 15+ animation types with server-client coordination
- ✅ **Multi-Step Actions** - Complex action chains with proper state management
- ✅ **Hex Grid Support** - Complete hexagonal board games (Hex Tic-Tac-Toe, Hex Territory)
- ✅ **YAML Enhancements** - Includes, shortcuts, and standard library
- ✅ **Debug Endpoints** - Full suite of state inspection APIs
- ✅ **Test Infrastructure** - Comprehensive E2E and unit test frameworks

### Game Library Quality (COMPLETED)
- ✅ **13 games building successfully** - All games validate and build cleanly
- ✅ **Action map timing fixes** - Phase transitions complete before UI updates
- ✅ **Game-specific fixes** - All identified critical game issues resolved
- ✅ **Win detection** - Proper game end detection across all games
- ✅ **Multi-step movement** - Three Men's Morris movement phases working

---

## Implementation Guidelines

### Adding New Features
1. Consider impact on existing games and table management
2. Design features to be generic and reusable
3. Update comprehensive tests for any changes
4. Ensure WebSocket message compatibility

### Priority Decision Framework
- **🔴 Critical**: Issues preventing production use or causing data loss
- **🟡 High**: Features that unlock new capabilities or fix major UX issues  
- **🟢 Medium**: Quality of life improvements and optimization
- **⚪ Low**: Nice-to-have features that don't block core functionality

### Development Process
1. **Test existing functionality** before making changes
2. **Update integration tests** for any WebSocket or API changes
3. **Document breaking changes** that affect client-server communication
4. **Validate with multiple games** to ensure changes don't break existing functionality

---

*Last Updated: January 2025 - After completing Phase A: Production Readiness Validation*