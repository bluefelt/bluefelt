# Building Clients

Building client applications for Bluefelt games involves creating real-time interfaces that connect to the Bluefelt server via WebSocket. This guide covers the React client implementation, protocols, and best practices for creating responsive game interfaces.

## Overview

Bluefelt clients are responsible for:
- **Real-time Communication** - WebSocket connection to game server
- **State Synchronization** - Applying JSON patches to maintain game state
- **Interactive UI** - Rendering game zones and handling player actions
- **Visual Feedback** - Showing game status, turn indicators, and animations

## Architecture

### Client-Server Communication

```
┌─────────────────┐    WebSocket     ┌─────────────────┐
│   React Client  │◄────────────────►│  Bluefelt       │
│                 │   JSON Patches   │  Server         │
└─────────────────┘                  └─────────────────┘
         │                                     │
         ▼                                     ▼
┌─────────────────┐                  ┌─────────────────┐
│   Local State   │                  │  Authoritative  │
│   (Zustand)     │                  │  Game State     │
└─────────────────┘                  └─────────────────┘
```

### Technology Stack

The reference React client uses:
- **React 18** - Component-based UI framework
- **TypeScript** - Type safety and developer experience
- **Zustand** - Lightweight state management
- **Vite** - Fast development and build tool
- **Tailwind CSS** - Utility-first styling
- **TanStack Query** - Server state management

## Getting Started

### Project Setup

```bash
cd clients/react
pnpm install
pnpm dev  # Start development server on port 5173
```

### Project Structure

```
clients/react/
├── src/
│   ├── components/          # Reusable UI components
│   │   ├── GameView.tsx     # Main game interface
│   │   ├── GameZones.tsx    # Zone rendering system
│   │   ├── Card.tsx         # Card component
│   │   ├── Board.tsx        # Grid board component
│   │   └── ...
│   ├── context/             # React contexts
│   │   ├── PlayerContext.tsx    # Player state
│   │   └── WebSocketContext.tsx # WebSocket connection
│   ├── ws/                  # WebSocket utilities
│   │   ├── useWebSocket.ts      # WebSocket hook
│   │   ├── useLobbyWebSocket.ts # Lobby-specific logic
│   │   └── useReconnectingWebSocket.ts # Auto-reconnection
│   ├── types/               # TypeScript definitions
│   │   └── messages.ts      # WebSocket message types
│   ├── utils/               # Utility functions
│   │   └── entityUtils.ts   # Entity manipulation helpers
│   └── pages/               # Application pages
│       ├── HomePage.tsx     # Game lobby list
│       ├── LobbyPage.tsx    # Individual game lobby
│       └── LoginPage.tsx    # Player identification
└── public/                  # Static assets
    └── tokens/              # Game piece graphics
```

## WebSocket Protocol

### Message Types

#### Client to Server

```typescript
// Action message - player performs game action
interface ActionMessage {
  type: 'action';
  data: {
    verb: string;           // Action verb (place, move, etc.)
    args: Record<string, any>; // Action arguments
  };
}

// Example: Place piece on board
{
  type: 'action',
  data: {
    verb: 'place',
    args: {
      location: '/zones/board/cells/0/0',
      entity: 'x_token'
    }
  }
}
```

#### Server to Client

```typescript
// Welcome message - initial game state
interface WelcomeMessage {
  type: 'welcome';
  data: {
    playerId: string;       // Assigned player ID
    state: GameState;       // Complete game state
    tick: number;           // Current state version
  };
}

// Diff message - state changes
interface DiffMessage {
  type: 'diff';
  data: {
    tick: number;           // New state version
    patches: JsonPatch[];   // JSON patch operations
  };
}

// Error message - action rejected
interface ErrorMessage {
  type: 'error';
  data: {
    message: string;        // Error description
    code?: string;          // Error code
  };
}
```

### Connection Management

```typescript
// WebSocket hook with auto-reconnection
export function useReconnectingWebSocket(url: string) {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [reconnectAttempts, setReconnectAttempts] = useState(0);
  
  const connect = useCallback(() => {
    try {
      const ws = new WebSocket(url);
      
      ws.onopen = () => {
        setIsConnected(true);
        setReconnectAttempts(0);
        console.log('WebSocket connected');
      };
      
      ws.onclose = () => {
        setIsConnected(false);
        setSocket(null);
        
        // Auto-reconnect with exponential backoff
        const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
        setTimeout(() => {
          setReconnectAttempts(prev => prev + 1);
          connect();
        }, delay);
      };
      
      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
      
      setSocket(ws);
    } catch (error) {
      console.error('Failed to create WebSocket:', error);
    }
  }, [url, reconnectAttempts]);
  
  useEffect(() => {
    connect();
    return () => socket?.close();
  }, [connect]);
  
  return { socket, isConnected };
}
```

## State Management

For a comprehensive guide on state structure and synchronization between client and server, see [State Structure and Synchronization](./state-structure.md).

### Game State Structure

```typescript
interface LobbyState {
  game: {
    zones: Record<string, Zone>;    // All game zones
    tick: number;                   // State version counter
    turn: number;                   // Current turn number
    currentPlayer: string;          // Active player ID
    gameStatus: {
      state: 'playing' | 'ended';   // Game phase
      winner?: string;              // Winner player ID
      tie: boolean;                 // Tie game flag
    };
    phases?: Record<string, string>; // Current phases
  };
  ui: {
    players: string[];              // Player usernames
    actionMap?: Record<string, ActionInfo>; // Available actions
    entities?: Entity[];            // Entity definitions
    zones?: Zone[];                 // Zone metadata
  };
  you: string;                      // Your player ID
  started: boolean;                 // Game started flag
}

interface Zone {
  type: 'grid' | 'list' | 'deck';
  cells?: (Entity | null)[][];    // Grid zone contents
  items?: Entity[];               // List/deck contents
}

interface Entity {
  entity: string;                 // Entity type ID
  props?: Record<string, any>;    // Entity properties
}
```

### State Synchronization

```typescript
// Apply JSON patches to update state
function applyPatches(state: GameState, patches: JsonPatch[]): GameState {
  let newState = { ...state };
  
  for (const patch of patches) {
    switch (patch.op) {
      case 'replace':
        newState = setValueAtPath(newState, patch.path, patch.value);
        break;
      case 'add':
        newState = addValueAtPath(newState, patch.path, patch.value);
        break;
      case 'remove':
        newState = removeValueAtPath(newState, patch.path);
        break;
    }
  }
  
  return newState;
}

// Helper: Set value at JSON pointer path
function setValueAtPath(obj: any, path: string, value: any): any {
  const parts = path.split('/').filter(p => p !== '');
  let current = { ...obj };
  let target = current;
  
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (Array.isArray(target[part])) {
      target[part] = [...target[part]];
    } else {
      target[part] = { ...target[part] };
    }
    target = target[part];
  }
  
  const lastPart = parts[parts.length - 1];
  if (Array.isArray(target) && !isNaN(Number(lastPart))) {
    target[Number(lastPart)] = value;
  } else {
    target[lastPart] = value;
  }
  
  return current;
}
```

### State Store

```typescript
// Zustand store for lobby state
interface LobbyStore {
  // State
  lobbyState: LobbyState | null;
  isConnected: boolean;
  
  // Actions
  setLobbyState: (state: LobbyState) => void;
  applyPatches: (patches: JsonPatch[]) => void;
  setConnectionStatus: (connected: boolean) => void;
  
  // Selectors
  getZone: (zoneId: string) => Zone | null;
  getCurrentPlayerEntity: () => string | null;
  getAvailableActions: () => Record<string, ActionInfo>;
  isCurrentPlayerTurn: () => boolean;
}

export const useLobbyStore = create<LobbyStore>((set, get) => ({
  lobbyState: null,
  isConnected: false,
  
  setLobbyState: (state) => set({ lobbyState: state }),
  
  applyPatches: (patches) => set((prev) => ({
    lobbyState: prev.lobbyState ? applyPatches(prev.lobbyState, patches) : null
  })),
  
  setConnectionStatus: (connected) => set({ isConnected: connected }),
  
  getZone: (zoneId) => {
    const state = get().lobbyState;
    return state?.game?.zones[zoneId] || null;
  },
  
  getCurrentPlayerEntity: () => {
    const { lobbyState } = get();
    return lobbyState?.you ? `token_${lobbyState.you}` : null;
  },
  
  getAvailableActions: () => {
    const { lobbyState } = get();
    return lobbyState?.ui?.actionMap?.[lobbyState.you] || {};
  },
  
  isCurrentPlayerTurn: () => {
    const { lobbyState } = get();
    return lobbyState?.game?.currentPlayer === lobbyState?.you;
  }
}));
```

## UI Components

### Game View Component

```typescript
// Main game interface
export function GameView() {
  const { lobbyState, isConnected } = useLobbyStore();
  const { sendAction } = useWebSocket();
  
  if (!lobbyState) {
    return <div>Loading game...</div>;
  }
  
  return (
    <div className="game-view">
      <GameHeader lobbyState={lobbyState} />
      <TurnIndicator 
        currentPlayer={lobbyState.game?.currentPlayer}
        isConnected={isConnected}
      />
      <GameZones 
        zones={lobbyState.game?.zones}
        actionMap={lobbyState.ui?.actionMap?.[lobbyState.you] || {}}
        onAction={sendAction}
      />
      <GameLog />
      {lobbyState.game?.gameStatus?.state === 'ended' && (
        <GameEndDisplay 
          winner={lobbyState.game.gameStatus.winner}
          tie={lobbyState.game.gameStatus.tie}
        />
      )}
    </div>
  );
}
```

### Interactive Zones

```typescript
// Zone rendering system
export function GameZones({ 
  zones, 
  actionMap, 
  onAction 
}: {
  zones: Record<string, Zone>;
  actionMap: Record<string, ActionInfo>;
  onAction: (verb: string, args: any) => void;
}) {
  return (
    <div className="game-zones">
      {Object.entries(zones).map(([zoneId, zone]) => (
        <ZoneComponent
          key={zoneId}
          zoneId={zoneId}
          zone={zone}
          actionMap={actionMap}
          onAction={onAction}
        />
      ))}
    </div>
  );
}

// Individual zone component
function ZoneComponent({ zoneId, zone, actionMap, onAction }: ZoneProps) {
  switch (zone.type) {
    case 'grid':
      return (
        <GridZone
          zoneId={zoneId}
          zone={zone}
          actionMap={actionMap}
          onAction={onAction}
        />
      );
    case 'list':
    case 'deck':
      return (
        <ListZone
          zoneId={zoneId}
          zone={zone}
          actionMap={actionMap}
          onAction={onAction}
        />
      );
    default:
      return <div>Unknown zone type</div>;
  }
}
```

### Grid Zone (Board)

```typescript
// Grid-based game board
export function GridZone({ zoneId, zone, actionMap, onAction }: GridZoneProps) {
  const handleCellClick = (row: number, col: number) => {
    const location = `/zones/${zoneId}/cells/${row}/${col}`;
    const action = actionMap[location];
    
    if (action) {
      onAction(action.action, {
        location,
        entity: getCurrentPlayerEntity()
      });
    }
  };
  
  return (
    <div className="grid-zone">
      <h3>{zoneId}</h3>
      <div className="grid-board">
        {zone.cells?.map((row, rowIndex) => (
          <div key={rowIndex} className="grid-row">
            {row.map((cell, colIndex) => (
              <InteractiveCell
                key={`${rowIndex}-${colIndex}`}
                cell={cell}
                row={rowIndex}
                col={colIndex}
                isInteractive={`/zones/${zoneId}/cells/${rowIndex}/${colIndex}` in actionMap}
                onClick={() => handleCellClick(rowIndex, colIndex)}
              />
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}

// Interactive grid cell
function InteractiveCell({ 
  cell, 
  row, 
  col, 
  isInteractive, 
  onClick 
}: InteractiveCellProps) {
  return (
    <div
      className={`grid-cell ${isInteractive ? 'interactive' : ''} ${cell ? 'occupied' : 'empty'}`}
      onClick={isInteractive ? onClick : undefined}
    >
      {cell && <EntityComponent entity={cell} />}
    </div>
  );
}
```

### Entity Rendering

```typescript
// Entity visual representation
export function EntityComponent({ entity }: { entity: Entity }) {
  const entityDef = getEntityDefinition(entity.entity);
  
  if (!entityDef) {
    return <div className="unknown-entity">{entity.entity}</div>;
  }
  
  // Render based on entity type
  switch (entityDef.type) {
    case 'token':
      return <TokenComponent entity={entity} definition={entityDef} />;
    case 'card':
      return <CardComponent entity={entity} definition={entityDef} />;
    case 'piece':
      return <PieceComponent entity={entity} definition={entityDef} />;
    default:
      return <div className="generic-entity">{entity.entity}</div>;
  }
}

// Token component (for tic-tac-toe, checkers, etc.)
function TokenComponent({ entity, definition }: EntityComponentProps) {
  const tokenType = definition.ui?.tokenType || 'token_circle';
  const color = definition.ui?.color || '#888888';
  
  return (
    <div 
      className={`token ${tokenType}`}
      style={{ color }}
    >
      <img src={`/tokens/${tokenType}.svg`} alt={entity.entity} />
    </div>
  );
}

// Card component (for card games)
function CardComponent({ entity, definition }: EntityComponentProps) {
  const { suit, rank } = definition.props || {};
  
  return (
    <div className={`playing-card ${suit}`}>
      <div className="card-rank">{rank}</div>
      <div className="card-suit">{getSuitSymbol(suit)}</div>
    </div>
  );
}
```

## Styling and Theming

### Tailwind CSS Classes

```css
/* Game layout */
.game-view {
  @apply flex flex-col h-screen bg-gray-100;
}

.game-zones {
  @apply flex-1 p-4 overflow-auto;
}

/* Grid board */
.grid-board {
  @apply inline-grid gap-1 border-2 border-gray-400 p-2 bg-white rounded;
}

.grid-cell {
  @apply w-16 h-16 border border-gray-300 flex items-center justify-center cursor-pointer transition-colors;
}

.grid-cell.interactive {
  @apply hover:bg-blue-100 border-blue-300;
}

.grid-cell.occupied {
  @apply cursor-default;
}

/* Tokens */
.token {
  @apply w-12 h-12 rounded-full flex items-center justify-center;
}

.token img {
  @apply w-full h-full;
}

/* Cards */
.playing-card {
  @apply w-16 h-24 bg-white border border-gray-400 rounded flex flex-col items-center justify-center text-lg font-bold;
}

.playing-card.hearts,
.playing-card.diamonds {
  @apply text-red-500;
}

.playing-card.clubs,
.playing-card.spades {
  @apply text-black;
}
```

### Dynamic Styling

```typescript
// Dynamic grid sizing
function getGridStyles(rows: number, cols: number) {
  return {
    gridTemplateRows: `repeat(${rows}, minmax(0, 1fr))`,
    gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))`,
  };
}

// Responsive design
function useResponsiveGrid(rows: number, cols: number) {
  const [cellSize, setCellSize] = useState(64);
  
  useEffect(() => {
    function updateSize() {
      const container = document.querySelector('.grid-container');
      if (container) {
        const containerWidth = container.clientWidth;
        const containerHeight = container.clientHeight;
        
        const maxCellWidth = Math.floor(containerWidth / cols) - 8;
        const maxCellHeight = Math.floor(containerHeight / rows) - 8;
        
        setCellSize(Math.min(maxCellWidth, maxCellHeight, 80));
      }
    }
    
    updateSize();
    window.addEventListener('resize', updateSize);
    return () => window.removeEventListener('resize', updateSize);
  }, [rows, cols]);
  
  return cellSize;
}
```

## Advanced Features

### Animations

```typescript
// Smooth state transitions
import { useSpring, animated } from '@react-spring/web';

function AnimatedEntity({ entity, position }: AnimatedEntityProps) {
  const springProps = useSpring({
    transform: `translate(${position.x}px, ${position.y}px)`,
    config: { tension: 300, friction: 30 }
  });
  
  return (
    <animated.div style={springProps}>
      <EntityComponent entity={entity} />
    </animated.div>
  );
}

// Entity movement animation
function useEntityMovement(entity: Entity, zoneId: string) {
  const [position, setPosition] = useState({ x: 0, y: 0 });
  
  useEffect(() => {
    // Calculate position based on zone and entity location
    const newPosition = calculateEntityPosition(entity, zoneId);
    setPosition(newPosition);
  }, [entity, zoneId]);
  
  return position;
}
```

### Sound Effects

```typescript
// Audio feedback system
class AudioManager {
  private sounds: Record<string, HTMLAudioElement> = {};
  
  constructor() {
    this.loadSounds([
      'place-piece',
      'move-piece', 
      'win-game',
      'turn-change'
    ]);
  }
  
  private loadSounds(soundNames: string[]) {
    soundNames.forEach(name => {
      this.sounds[name] = new Audio(`/sounds/${name}.mp3`);
    });
  }
  
  play(soundName: string) {
    const sound = this.sounds[soundName];
    if (sound) {
      sound.currentTime = 0;
      sound.play().catch(console.error);
    }
  }
}

export const audioManager = new AudioManager();

// Use in components
function useGameAudio() {
  const lobbyState = useLobbyStore(state => state.lobbyState);
  const prevTick = useRef(0);
  
  useEffect(() => {
    if (lobbyState && lobbyState.game.tick > prevTick.current) {
      audioManager.play('turn-change');
      prevTick.current = lobbyState.game.tick;
    }
  }, [lobbyState?.game?.tick]);
}
```

### Accessibility

```typescript
// Keyboard navigation
function useKeyboardNavigation(gridSize: { rows: number; cols: number }) {
  const [focusedCell, setFocusedCell] = useState({ row: 0, col: 0 });
  
  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      switch (event.key) {
        case 'ArrowUp':
          setFocusedCell(prev => ({
            ...prev,
            row: Math.max(0, prev.row - 1)
          }));
          break;
        case 'ArrowDown':
          setFocusedCell(prev => ({
            ...prev,
            row: Math.min(gridSize.rows - 1, prev.row + 1)
          }));
          break;
        case 'ArrowLeft':
          setFocusedCell(prev => ({
            ...prev,
            col: Math.max(0, prev.col - 1)
          }));
          break;
        case 'ArrowRight':
          setFocusedCell(prev => ({
            ...prev,
            col: Math.min(gridSize.cols - 1, prev.col + 1)
          }));
          break;
        case 'Enter':
        case ' ':
          // Trigger action at focused cell
          handleCellAction(focusedCell.row, focusedCell.col);
          break;
      }
    }
    
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [gridSize, focusedCell]);
  
  return focusedCell;
}

// Screen reader support
function GridCell({ 
  cell, 
  row, 
  col, 
  isInteractive, 
  isFocused,
  onClick 
}: GridCellProps) {
  return (
    <div
      role="gridcell"
      tabIndex={isFocused ? 0 : -1}
      aria-label={`Cell ${row + 1}, ${col + 1}${cell ? `, contains ${cell.entity}` : ', empty'}`}
      aria-disabled={!isInteractive}
      className={`grid-cell ${isInteractive ? 'interactive' : ''} ${isFocused ? 'focused' : ''}`}
      onClick={isInteractive ? onClick : undefined}
    >
      {cell && <EntityComponent entity={cell} />}
    </div>
  );
}
```

## Performance Optimization

### Memoization

```typescript
// Prevent unnecessary re-renders
const MemoizedGameZones = memo(function GameZones({ zones, actionMap, onAction }) {
  return (
    <div className="game-zones">
      {Object.entries(zones).map(([zoneId, zone]) => (
        <MemoizedZone
          key={zoneId}
          zoneId={zoneId}
          zone={zone}
          actionMap={actionMap}
          onAction={onAction}
        />
      ))}
    </div>
  );
});

// Selective subscriptions
function useZoneState(zoneId: string) {
  return useLobbyStore(
    useCallback(
      (state) => state.lobbyState?.game?.zones[zoneId],
      [zoneId]
    ),
    shallow
  );
}
```

### Virtual Scrolling

```typescript
// For large lists/decks
import { FixedSizeList as List } from 'react-window';

function VirtualizedCardList({ cards }: { cards: Entity[] }) {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => (
    <div style={style}>
      <CardComponent entity={cards[index]} />
    </div>
  );
  
  return (
    <List
      height={400}
      itemCount={cards.length}
      itemSize={120}
      width="100%"
    >
      {Row}
    </List>
  );
}
```

## Testing

### Unit Testing

```typescript
// Test game state transformations
import { applyPatches } from '../utils/stateUtils';

describe('State Management', () => {
  test('applies place action patch correctly', () => {
    const initialState = {
      game: {
        zones: {
          board: {
            type: 'grid',
            cells: [[null, null], [null, null]]
          }
        },
        tick: 0
      }
    };
    
    const patches = [{
      op: 'replace',
      path: '/game/zones/board/cells/0/0',
      value: { entity: 'x_token' }
    }];
    
    const newState = applyPatches(initialState, patches);
    
    expect(newState.game.zones.board.cells[0][0]).toEqual({
      entity: 'x_token'
    });
  });
});
```

### Integration Testing

```typescript
// Test WebSocket integration
import { renderHook, act } from '@testing-library/react';
import { MockWebSocket } from 'mock-websocket';

describe('WebSocket Integration', () => {
  test('handles welcome message correctly', async () => {
    const mockWS = new MockWebSocket('ws://localhost:8000/ws/lobby/test/p1');
    
    const { result } = renderHook(() => 
      useWebSocket('ws://localhost:8000/ws/lobby/test/p1')
    );
    
    act(() => {
      mockWS.send(JSON.stringify({
        type: 'welcome',
        data: {
          playerId: 'p1',
          state: { /* game state */ },
          tick: 0
        }
      }));
    });
    
    expect(result.current.playerId).toBe('p1');
  });
});
```

## Deployment

### Build Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          game: ['zustand', '@tanstack/react-query']
        }
      }
    }
  },
  server: {
    proxy: {
      '/ws': {
        target: 'ws://localhost:8000',
        ws: true
      },
      '/api': {
        target: 'http://localhost:8000'
      }
    }
  }
});
```

### Environment Configuration

```typescript
// Environment-specific settings
const config = {
  development: {
    wsUrl: 'ws://localhost:8000',
    apiUrl: 'http://localhost:8000'
  },
  production: {
    wsUrl: process.env.VITE_WS_URL || 'wss://api.bluefelt.com',
    apiUrl: process.env.VITE_API_URL || 'https://api.bluefelt.com'
  }
};

export const getConfig = () => {
  const env = process.env.NODE_ENV || 'development';
  return config[env];
};
```

Building Bluefelt clients requires understanding the real-time WebSocket protocol, efficient state management, and creating responsive game interfaces. The React implementation provides a solid foundation that can be adapted for different games and extended with additional features.