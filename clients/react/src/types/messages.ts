// WebSocket message types shared between client and server

export type WelcomeMessage = {
  type: 'welcome';
  you: string;
  started: boolean;
  state?: GameState;
  meta: MetaState;
  tick?: number;
};

export type PlayerUpdateMessage = {
  type: 'playerUpdate';
  players: string[];
};

export type GameStartedMessage = {
  type: 'gameStarted';
  you?: string;
  state: GameState;
  meta: MetaState;
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
  players: Array<{ id: string }>;
  turn: string;
  zones: Record<string, unknown[][]>;
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

export type MetaState = {
  actionMap?: Record<string, Record<string, ActionInfo>>;
  players?: string[];
  entities?: EntityDefinition[];
  gameStatus?: {
    state: 'ended';
    winner?: string;
    tie?: boolean;
  };
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