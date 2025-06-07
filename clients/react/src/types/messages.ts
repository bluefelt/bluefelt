// WebSocket message types shared between client and server

export type WelcomeMessage = {
  type: 'welcome';
  you: string;
  started: boolean;
  // Support both old and new formats during transition
  state?: GameState;
  meta?: MetaState;
  game?: GameState;
  ui?: MetaState;
  tick?: number;
};

export type PlayerUpdateMessage = {
  type: 'playerUpdate';
  players: string[];
};

export type GameStartedMessage = {
  type: 'gameStarted';
  you?: string;
  // Support both old and new formats during transition
  state?: GameState;
  meta?: MetaState;
  game?: GameState;
  ui?: MetaState;
};

export type DiffMessage = {
  type: 'diff';
  tick: number;
  patch: PatchOperation[];
};

export type InfoMessage = {
  type: 'info';
  message: string;
};

export type ErrorMessage = {
  type: 'error';
  message: string;
};

export type StartedMessage = {
  type: 'started';
};

export type GameState = {
  players?: Array<{ id: string }>;
  zones: Record<string, any>; // Can be arrays or objects with type/cells
  currentPlayer?: string;
  turn?: number;
  tick?: number;
  gameStatus?: {
    state: 'playing' | 'ended';
    winner?: string | null;
    tie?: boolean;
  };
  phases?: any;
  selection?: any;
  temp?: any;
};

export type EntityDefinition = {
  id: string;
  props?: {
    value?: string;
  };
  ui?: {
    glyph?: string;
    tokenType?: string;
  };
};

export type ZoneGroup = {
  id: string;
  title: string;
  zones: string[];
};

export type MetaState = {
  actionMap?: Record<string, Record<string, ActionInfo>>;
  players?: string[];
  entities?: EntityDefinition[];
  gameStatus?: {
    state: 'playing' | 'ended';
    winner?: string | null;
    tie?: boolean;
  };
  zoneGroups?: ZoneGroup[];
  currentPhasePrompt?: string;
  phaseDisplayMessages?: any[];
  zones?: any;
  turn?: number;
  currentPlayer?: string;
  tick?: number;
  gameLog?: Array<{
    player: string;
    actor: string;
    message: string;
    timestamp: string;
  }>;
  phaseStates?: any;
};

export type ActionInfo = {
  action: string;
  direction: string;
};

export type PatchOperation = {
  op: 'add' | 'remove' | 'replace';
  path: string;
  value?: unknown;
};

export type ServerMessage = 
  | WelcomeMessage
  | PlayerUpdateMessage
  | GameStartedMessage
  | DiffMessage
  | InfoMessage
  | ErrorMessage
  | StartedMessage;

export type ClientAction = 
  | { action: 'join' }
  | { action: 'leave' }
  | { action: 'start_game' }
  | { action: string; args: { row: number; col: number } };

// Additional types for testing
export type Phase = any;
export type Zone = any;
export type Entity = any;

export type LobbyState = {
  you?: string;
  started?: boolean;
  game?: GameState;
  ui?: MetaState;
  // Legacy properties for backward compatibility
  state?: GameState;
  meta?: MetaState;
};