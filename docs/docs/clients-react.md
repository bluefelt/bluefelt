# Official Client: React

The React client is Bluefelt's reference implementation, demonstrating how to build a modern web-based game client. Built with React 18 and TypeScript, it showcases best practices for creating responsive, accessible game interfaces.

## Overview

The React client serves as:
- **Reference Implementation** - Shows how clients should interact with the server
- **Testing Ground** - First client to support new features
- **Production Ready** - Fully functional for playing games
- **Learning Resource** - Well-documented code for other client developers

## Technology Stack

- **React 18** - Modern React with hooks and concurrent features
- **TypeScript** - Full type safety for robust development
- **Vite** - Lightning-fast development and build tool
- **Tailwind CSS** - Utility-first styling system
- **Vitest** - Fast unit and integration testing
- **pnpm** - Efficient package management

## Getting Started

### Prerequisites

- Node.js 18+
- pnpm (install with `npm install -g pnpm`)

### Installation

```bash
cd clients/react
pnpm install
```

### Development

```bash
# Start development server (port 5173)
pnpm dev

# Run tests
pnpm test

# Run tests with UI
pnpm test:ui

# Build for production
pnpm build

# Preview production build
pnpm preview
```

## Project Structure

```
clients/react/
├── src/
│   ├── components/        # UI components
│   │   ├── zones/        # Zone-specific components
│   │   │   ├── BoardZone.tsx      # Grid rendering
│   │   │   ├── CardZone.tsx       # List/hand rendering
│   │   │   ├── ChoiceZone.tsx     # Choice rendering
│   │   │   └── HexBoard.tsx       # Hex grid support
│   │   ├── GameView.tsx           # Main game container
│   │   ├── GameZones.tsx          # Zone orchestration
│   │   └── Card.tsx               # Card component
│   ├── context/          # React contexts
│   │   ├── PlayerContext.tsx      # Player identity
│   │   ├── WebSocketContext.tsx   # WS connection
│   │   └── AnimationContext.tsx   # Animation settings
│   ├── hooks/            # Custom React hooks
│   │   ├── useGameActions.ts      # Action handling
│   │   └── useCardStyle.ts        # Card theming
│   ├── ws/              # WebSocket utilities
│   │   └── useLobbyWebSocket.ts   # Lobby connection
│   ├── types/           # TypeScript definitions
│   ├── utils/           # Helper functions
│   └── pages/           # Route components
```

## Key Features

### 1. Lobby Management

The client supports comprehensive lobby management with ownership and archival:

```typescript
// Lobby page with ownership controls
{lobbyState.owner === player?.username && (
  <button onClick={() => setIsEditingName(true)}>
    <svg className="w-4 h-4">...</svg>
  </button>
)}

// Archived lobby warning
{lobbyState.archived && (
  <div className="bg-red-500 bg-opacity-10 border border-red-500 rounded p-3">
    <p className="text-red-400 font-semibold">
      This lobby is archived and cannot be joined.
    </p>
  </div>
)}
```

### 2. Generic Zone Rendering

The client dynamically renders any zone type without game-specific code:

```typescript
// GameZones component handles any zone type
export function GameZones({ zones, actionMap }) {
  return (
    <>
      {Object.entries(zones).map(([zoneId, zone]) => {
        switch (zone.type) {
          case 'grid':
            return <BoardZone key={zoneId} zone={zone} />;
          case 'list':
          case 'hand':
            return <CardZone key={zoneId} zone={zone} />;
          case 'choice':
            return <ChoiceZone key={zoneId} zone={zone} />;
          default:
            return <div>Unknown zone type: {zone.type}</div>;
        }
      })}
    </>
  );
}
```

### 3. Smart Action Detection

Components automatically detect available actions from the server:

```typescript
// Detect column-based actions (Connect 4 style)
const columnActions = Object.keys(actionMap || {})
  .filter(path => path.includes(`/zones/${zoneId}/columns/`))
  .map(path => {
    const match = path.match(/\/columns\/(\d+)/);
    return match ? parseInt(match[1]) : -1;
  });

// Render column drop zones if detected
{columnActions.length > 0 && (
  <ColumnDropZones columns={columnActions} />
)}
```

### 3. State Synchronization

Efficient state updates using JSON patches with table support:

```typescript
function useLobbyWebSocket(lobbyId: string) {
  const handleMessage = useCallback((event: MessageEvent) => {
    const data = JSON.parse(event.data);
    
    switch (data.type) {
      case 'lobbyJoined':
        setLobbyState(data.lobby);
        break;
      case 'tableCreated':
        setLobbyState(prev => ({
          ...prev,
          tables: [...prev.tables, data.table]
        }));
        break;
      case 'tableUpdated':
        // Update specific table state
        setLobbyState(prev => ({
          ...prev,
          tables: prev.tables.map(t => 
            t.id === data.tableId ? { ...t, ...data } : t
          )
        }));
        break;
      case 'gameUpdate':
        if (data.patches) {
          setGameState(prev => applyPatches(prev, data.patches));
        }
        break;
    }
  }, []);
  
  // ... Enhanced WebSocket with table operations
}
```

### Lobby Management Features

**Lobby State Structure**:
```typescript
type LobbyState = {
  id: string;
  name: string;
  owner?: string | null;    // Username of lobby owner
  archived?: boolean;       // Whether lobby is archived
  members: LobbyMember[];
  tables: Table[];
  recentChat: ChatMessage[];
  game?: GameInstance;      // Current active game
  you?: string;             // Your player ID
  error?: string;
  isJoined?: boolean;       // Whether user has joined lobby
};
```

**Lobby Actions**:
```typescript
const {
  lobbyState,
  connected,
  joinLobby,        // Join the lobby (separate from viewing)
  renameLobby,      // Rename lobby (owner only)
  leaveLobby,       // Leave lobby
  createTable,      // Create a new table
  joinTable,        // Join a table (auto-seat assignment)
  sendGameAction,   // Send game actions
} = useLobbyWebSocket(lobbyId, username);
```

**Ownership and Archival**:
- The first player to join a lobby becomes the owner
- Owners can rename the lobby using inline editing
- When the owner leaves, ownership transfers to the earliest joined remaining member
- When all members leave, the lobby is archived and cannot be rejoined
- Archived lobbies are hidden from the public lobby list

### 5. Responsive Design

Adapts to different screen sizes and devices:

```typescript
// Responsive grid sizing
const cellSize = useMemo(() => {
  const maxWidth = containerWidth / cols - gap;
  const maxHeight = containerHeight / rows - gap;
  return Math.min(maxWidth, maxHeight, 100); // Cap at 100px
}, [containerWidth, containerHeight, rows, cols]);
```

### 6. Animation System

The React client features a sophisticated animation system that responds to server-provided hints while maintaining smooth, responsive gameplay.

#### Architecture

The animation system consists of three main components:

**1. PatchAnalyzer** - Detects animatable changes in JSON patches
```typescript
// Analyzes patches to create animation plans
const analyzer = new PatchAnalyzer();
const animationPlan = analyzer.analyzeForAnimation({
  patch: jsonPatch,
  currentState: gameState,
  currentPlayer: 'p1'
});
```

**2. AnimationEngine** - Orchestrates and executes animations
```typescript
// Processes patches and queues animations
const engine = new AnimationEngine({
  onAnimationStart: (animation) => { /* track */ },
  onAnimationComplete: (result) => { /* cleanup */ }
});

await engine.processAnimatablePatch(patch, state, config);
```

**3. AnimationContext** - Manages user preferences
```typescript
const AnimationContext = createContext({
  enableAnimations: true,
  animationSpeed: 1.0,
  enableSounds: true,
  reduceMotion: false
});
```

#### Server Animation Hints

The server can provide animation metadata via the `_animation` field in patches:

```typescript
// Server patch with animation hint
{
  "op": "replace",
  "path": "/zones/board/cells/5/3",
  "value": { "entity": "disc_p1" },
  "_animation": {
    "type": "entity_spawn",
    "duration": 600,
    "isGravityDrop": true,
    "fromPosition": { "row": 0, "col": 3 },
    "toPosition": { "row": 5, "col": 3 }
  }
}
```

#### Animation Types

The system supports various animation types:

- **Entity Movement** - Pieces moving between positions
- **Entity Spawn** - New pieces appearing
- **Entity Destroy** - Pieces being removed
- **Zone Transfer** - Cards moving between zones
- **Selection Change** - Highlighting selected items
- **Turn Change** - Visual turn indicators
- **Phase Change** - Game phase transitions
- **Game End** - Victory/defeat celebrations

#### Gravity Drop Animation (Connect 4)

Special handling for gravity-based games:

```typescript
// Gravity drop with bounce effect
if (metadata.isGravityDrop) {
  const dropDistance = (toRow - fromRow) * cellHeight;
  
  const keyframes = [
    { transform: `translateY(-${dropDistance}px) scale(0.8)`, opacity: '0' },
    { transform: `translateY(-${dropDistance}px) scale(1)`, opacity: '1', offset: 0.1 },
    { transform: 'translateY(0) scale(1)', opacity: '1', offset: 0.8 },
    { transform: 'translateY(-10px) scale(1.05)', opacity: '1', offset: 0.9 },
    { transform: 'translateY(0) scale(1)', opacity: '1' }
  ];
  
  element.animate(keyframes, {
    duration: duration * 1.5,
    easing: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)'
  });
}
```

#### Retry Logic for DOM Timing

The animation system includes robust retry logic to handle React's asynchronous rendering:

```typescript
// Wait for element to be rendered
let targetElement = null;
for (let i = 0; i < 10; i++) {
  targetElement = findElementByPath(animation.targetPath);
  if (targetElement) break;
  await new Promise(resolve => setTimeout(resolve, 20));
}
```

#### Performance Considerations

- Animations are queued and processed sequentially
- Duplicate animations are automatically deduplicated
- CSS transforms used for optimal performance
- Animations can be disabled for accessibility
- Speed can be adjusted via user preferences

### 7. Table Management System

The client supports the full table lifecycle:

#### Table Creation
```typescript
<TableList
  tables={lobbyState.tables}
  currentPlayerId={username}
  onCreateTable={(bundleId, name) => {
    createTable(bundleId, name);
  }}
  onClaimSeat={claimSeat}
  onReleaseSeat={releaseSeat}
  onSetReady={setReady}
/>
```

#### Seat Status Display
```typescript
// Visual seat representation
<SeatSlot
  seat={seat}
  isReady={readyStates[index]}
  isCurrentPlayer={seat?.playerId === currentPlayerId}
  canClaim={!isSeatedAtAnyTable && !seat}
  canRelease={seat?.playerId === currentPlayerId}
/>
```

#### Countdown Timer
```typescript
// Real-time countdown display
{table.status === 'Countdown' && (
  <CountdownTimer 
    endsAt={table.countdownEndsAt} 
    onComplete={() => /* game starts */}
  />
)}
```

### 8. Dual-Scope Chat

Integrated chat with lobby and table contexts:

```typescript
<LobbyChat
  messages={recentChat}
  currentScope={userTable ? 'table' : 'lobby'}
  currentTableId={userTable?.id}
  currentUsername={username}
  onSendMessage={(message, scope, tableId) => {
    sendChatMessage(message, scope, tableId);
  }}
/>
```

### 9. Player Preferences

Customizable themes and settings:

```typescript
// Card style preferences
const cardStyles = {
  'classic': ClassicCardStyle,
  'modern': ModernCardStyle,
  'minimal': MinimalCardStyle
};

// Token preferences
const tokenSets = {
  'default': { x: '✕', o: '○' },
  'emoji': { x: '❌', o: '⭕' },
  'letters': { x: 'X', o: 'O' }
};
```

## Development Patterns

### Component Patterns

**1. Zone Components**

All zone components follow a consistent pattern:

```typescript
interface ZoneComponentProps {
  zoneId: string;
  zone: Zone;
  actionMap?: ActionMap;
  isMyTurn: boolean;
}

export function ZoneComponent({ 
  zoneId, 
  zone, 
  actionMap, 
  isMyTurn 
}: ZoneComponentProps) {
  // Detect available actions
  const actions = useZoneActions(zoneId, actionMap);
  
  // Render based on zone data
  return (
    <div className="zone">
      {/* Zone-specific rendering */}
    </div>
  );
}
```

**2. Entity Components**

Entities render based on their type:

```typescript
function EntityDisplay({ entity }: { entity: Entity }) {
  const entityDef = useEntityDefinition(entity.entity);
  
  if (entityDef?.type === 'card') {
    return <CardDisplay entity={entity} definition={entityDef} />;
  }
  
  if (entityDef?.type === 'token') {
    return <TokenDisplay entity={entity} definition={entityDef} />;
  }
  
  // Fallback for unknown entities
  return <div className="entity">{entity.entity}</div>;
}
```

### State Management

The client uses React Context for global state:

#### PlayerContext Architecture Change

**IMPORTANT**: PlayerContext is now a wrapper around PlayerPreferencesContext for consistency:

```typescript
// PlayerPreferencesContext (Core)
const PlayerPreferencesContext = createContext<{
  playerId: string | null;
  playerName: string | null;
  preferences: PlayerPreferences;
  setPlayer: (id: string, name: string) => void;
  updatePreferences: (prefs: Partial<PlayerPreferences>) => void;
}>({
  playerId: null,
  playerName: null,
  preferences: defaultPreferences,
  setPlayer: () => {},
  updatePreferences: () => {}
});

// PlayerContext (Wrapper for backward compatibility)
const PlayerContext = createContext<{
  playerId: string | null;
  playerName: string | null;
  setPlayer: (id: string, name: string) => void;
}>({
  playerId: null,
  playerName: null,
  setPlayer: () => {}
});
```

#### Required Provider Wrapper Hierarchy for Tests

**CRITICAL**: Tests must provide the correct provider wrapper hierarchy or components will fail:

```typescript
// Required test wrapper hierarchy (order matters!)
const RequiredTestProviders = ({ children }: { children: React.ReactNode }) => (
  <PlayerPreferencesProvider>
    <AnimationProvider>
      <WebSocketProvider>
        {children}
      </WebSocketProvider>
    </AnimationProvider>
  </PlayerPreferencesProvider>
);

// GameView integration requirements
const GameViewTestWrapper = ({ children }: { children: React.ReactNode }) => (
  <RequiredTestProviders>
    <div data-testid="game-container">
      {children}
    </div>
  </RequiredTestProviders>
);
```

#### Specific Requirements for GameView Tests

**Board Integration**: GameView tests must provide Board cells with specific data-testid format:
```typescript
// Board cells must have this exact pattern
<div 
  data-testid={`board-cell-${row}-${col}`} 
  onClick={() => handleCellClick(row, col)}
>
  {cellContent}
</div>
```

**WebSocket Context Structure**: Mock WebSocket context must include proper gameState structure:
```typescript
const mockWebSocketContext = {
  isConnected: true,
  sendAction: vi.fn(),
  gameState: {
    game: {
      zones: {
        board: {
          type: 'grid',
          cells: [[null, null, null], [null, null, null], [null, null, null]]
        }
      },
      currentPlayer: 'p1',
      turn: 0,
      gameStatus: { state: 'playing', winner: null }
    },
    ui: {
      actionMap: {
        p1: {
          '/zones/board/cells/0/0': {
            action: 'placeMark',
            direction: 'Click to place your marker'
          }
        }
      },
      players: ['alice', 'bob']
    },
    you: 'p1',
    started: true
  }
};
```

// WebSocket context for connection
const WebSocketContext = createContext<{
  isConnected: boolean;
  sendAction: (action: string, args: any) => void;
  gameState: GameState | null;
}>({
  isConnected: false,
  sendAction: () => {},
  gameState: null
});
```

### Testing Strategy

#### Provider Testing Patterns

**CRITICAL**: Tests that previously used PlayerContext must now use the correct provider hierarchy:

```typescript
// OLD pattern (will fail):
const TestWrapper = ({ children }) => (
  <PlayerContext.Provider value={mockPlayerValue}>
    {children}
  </PlayerContext.Provider>
);

// NEW pattern (required):
const TestWrapper = ({ children }) => (
  <PlayerPreferencesProvider>
    <AnimationProvider audioManager={mockAudioManager}>
      <WebSocketContext.Provider value={mockWebSocketContext}>
        {children}
      </WebSocketContext.Provider>
    </AnimationProvider>
  </PlayerPreferencesProvider>
);
```

**Common Test Setup Errors to Avoid:**

1. **Missing AnimationProvider**: Components using useAnimation will crash
2. **Wrong Provider Order**: Nested dependencies require specific order
3. **Incomplete Mock Context**: Missing required gameState structure
4. **Missing data-testid**: Components need specific test selectors

**Example Complete Test Setup:**
```typescript
// Complete test setup with all required providers
describe('GameComponent', () => {
  const mockAudioManager = {
    playSound: vi.fn(),
    preloadSounds: vi.fn().mockResolvedValue(undefined),
    setVolume: vi.fn()
  };
  
  const renderWithProviders = (component: React.ReactElement) => {
    return render(
      <PlayerPreferencesProvider>
        <AnimationProvider audioManager={mockAudioManager}>
          <WebSocketContext.Provider value={mockWebSocketContext}>
            {component}
          </WebSocketContext.Provider>
        </AnimationProvider>
      </PlayerPreferencesProvider>
    );
  };
  
  beforeEach(() => {
    // Mock audio APIs for test environment
    global.AudioContext = vi.fn(() => ({
      decodeAudioData: vi.fn().mockResolvedValue({}),
      createBufferSource: vi.fn(() => ({
        connect: vi.fn(),
        start: vi.fn(),
        buffer: null
      })),
      destination: {}
    }));
  });
  
  it('renders and handles interactions', () => {
    const { getByTestId } = renderWithProviders(<GameView />);
    expect(getByTestId('game-container')).toBeInTheDocument();
  });
});
```

Comprehensive test coverage ensures reliability:

```typescript
// Component testing
describe('BoardZone', () => {
  it('renders grid correctly', () => {
    const zone = {
      type: 'grid',
      cells: [[null, null], [null, null]]
    };
    
    const { container } = render(
      <BoardZone zone={zone} zoneId="board" />
    );
    
    expect(container.querySelectorAll('.cell')).toHaveLength(4);
  });
  
  it('handles click actions', async () => {
    const actionMap = {
      '/zones/board/cells/0/0': {
        action: 'place',
        args: { location: '/zones/board/cells/0/0' }
      }
    };
    
    const sendAction = vi.fn();
    const { getByTestId } = render(
      <BoardZone 
        zone={zone} 
        actionMap={actionMap}
        onAction={sendAction}
      />
    );
    
    fireEvent.click(getByTestId('cell-0-0'));
    expect(sendAction).toHaveBeenCalledWith('place', {
      location: '/zones/board/cells/0/0'
    });
  });
});
```

## Contributing

### Development Workflow

1. **Pick an Issue** - Check GitHub issues for tasks
2. **Create Branch** - `git checkout -b feature/your-feature`
3. **Write Tests** - Add tests for new functionality
4. **Implement** - Write clean, typed code
5. **Test** - Run `pnpm test` to ensure all tests pass
6. **Submit PR** - Create pull request with clear description

### Code Style

- Use TypeScript for all new code
- Follow existing patterns for consistency
- Add JSDoc comments for complex functions
- Keep components focused and small
- Use semantic HTML for accessibility

### Common Tasks

**Adding a New Zone Type:**

1. Create component in `src/components/zones/`
2. Add to zone type detection in `GameZones.tsx`
3. Write tests for the new component
4. Update types if needed

**Adding Animation:**

1. **For Server-Hinted Animations:**
   - Add `_animation` metadata to patches in server code
   - PatchAnalyzer will automatically detect and create animation plans
   - AnimationEngine will execute based on the metadata

2. **For Client-Side Animations:**
   - Use the AnimationContext for user preferences
   - Check if animations are enabled before applying
   - Use CSS transforms for best performance
   - Add appropriate easing functions

3. **Testing Animations:**
   - Test with animations enabled and disabled
   - Verify retry logic works for delayed DOM updates
   - Check performance on lower-end devices
   - Ensure animations complete even if interrupted

4. **Common Patterns:**
   ```typescript
   // Check animation preferences
   const { config } = useAnimation();
   if (!config.enableAnimations) return;
   
   // Apply animation with speed adjustment
   element.animate(keyframes, {
     duration: baseDuration / config.speed,
     easing: 'ease-out'
   });
   ```

**Improving Accessibility:**

1. Add ARIA labels and roles
2. Ensure keyboard navigation works
3. Test with screen readers
4. Check color contrast ratios

## Future Enhancements

The React client roadmap includes:

- **Progressive Web App** - Offline support and installability
- **Advanced Animations** - Smooth piece movements and transitions
- **Sound System** - Integrated audio feedback
- **Spectator Mode** - Watch games without playing
- **Tournament Support** - Multi-game competitions
- **Social Features** - Friends, chat, and profiles
- **Mobile App** - React Native version

## Getting Help

- **Documentation** - Check `/docs` for guides
- **Discord** - Join our community for quick help
- **GitHub Issues** - Report bugs or request features
- **Code Comments** - The codebase is well-documented

The React client demonstrates that a "dumb" client can still provide a rich, polished gaming experience while maintaining complete separation from game logic.