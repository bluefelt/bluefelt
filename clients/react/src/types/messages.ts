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

export type MetaState = {
  possibleVerbs?: Record<string, GroupedVerb[]>;
  players?: string[];
  gameStatus?: {
    state: 'ended';
    winner?: string;
    tie?: boolean;
  };
};

export type GroupedVerb = {
  verb: string;
  direction: string;
  validOptions: VerbOption[];
};

export type VerbOption = {
  zone: string;
  row: number;
  col: number;
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
  | { verb: string; args: { row: number; col: number } };