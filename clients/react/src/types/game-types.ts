/**
 * Type definitions for the new simplified game state structure
 */

// Lobby types
export interface LobbyMember {
  id: string;
  name: string;
  joinedAt: string;
}

export interface LobbySettings {
  maxMembers: number;
  allowObservers: boolean;
  autoStart: boolean;
  privateRoom: boolean;
}

export interface LobbyState {
  id: string;
  name: string;
  owner?: string | null;
  archived?: boolean;
  members: LobbyMember[];
  games: GameSummary[];
  completedGames: CompletedGame[];
  settings: LobbySettings;
}

export interface GameSummary {
  id: string;
  gameId: string;
  status: string;
  players: Record<string, string>;
  createdAt: string;
}

export interface CompletedGame {
  id: string;
  gameId: string;
  winner?: string;
  endedAt: string;
  duration: number;
}

// Game state types
export interface GameInstance {
  id: string;
  gameId: string;
  you: string; // Your player ID (p1, p2, etc.)
  players: Record<string, PlayerInfo>;
  state: GameState;
  ui: GameUI;
}

export interface PlayerInfo {
  id: string; // Username/member ID
  name?: string;
  connected?: boolean;
}

export interface GameState {
  currentPlayer: string;
  zones: Record<string, Zone>;
  phases: Record<string, string>;
  gameStatus: string; // "playing", "won:p1", "tie", "abandoned"
  selection: Record<string, any>;
  [key: string]: any; // Game-specific state
}

export interface Zone {
  // Grid zones
  cells?: (string | EntityData | null)[][];
  // List zones
  items?: (string | EntityData)[];
  // Zone metadata
  owner?: string;
  visibility?: string;
}

export interface EntityData {
  id: string;
  type?: string;
  owner?: string;
  [key: string]: any; // Entity-specific properties
}

// UI types
export interface GameUI {
  availableActions: Record<string, ActionDefinition>;
  gameLog: LogEntry[];
  zones: ZoneUI[];
  gameMetadata: GameMetadata;
}

export interface ActionDefinition {
  id: string;
  verb: string;
  label?: string;
  allowedZones?: string[];
  entityFilter?: any;
  conditions?: any[];
}

export interface LogEntry {
  message: string;
  type: string;
  timestamp: string;
}

export interface ZoneUI {
  id: string;
  display: string;
  shape?: string;
  tier?: number;
  position?: any;
}

export interface GameMetadata {
  name: string;
  description?: string;
  minPlayers: number;
  maxPlayers: number;
  estimatedTime?: string;
}

// Entity UI types
export interface EntityUI {
  zone: string;
  position?: [number, number];
  interactions: EntityInteraction[];
  isOwned?: boolean;
  owner?: string;
  isSelected?: boolean;
  displayStyle?: string;
  faceUp?: boolean;
}

export interface EntityInteraction {
  actionId: string;
  verb: string;
  label: string;
  style: InteractionStyle;
}

export interface InteractionStyle {
  highlight: string;
  cursor: string;
}

// WebSocket message types
export interface WSMessage {
  type: string;
  [key: string]: any;
}

export interface GameUpdateMessage extends WSMessage {
  type: 'gameUpdate';
  gameInstanceId: string;
  tick: number;
  patches: any[];
  entityUI: Record<string, Record<string, EntityUI>>;
}

export interface LobbyStateMessage extends WSMessage {
  type: 'lobbyState';
  lobby: LobbyState;
}

export interface ErrorMessage extends WSMessage {
  type: 'error';
  message: string;
}

// Action request types
export interface EntityInteractionRequest {
  action: 'entityInteraction';
  gameInstanceId: string;
  entityId: string;
  actionId: string;
  additionalArgs?: any;
}

export interface ZoneInteractionRequest {
  action: 'zoneInteraction';
  gameInstanceId: string;
  zoneId: string;
  position?: [number, number];
  actionId: string;
  additionalArgs?: any;
}

// Table/Seat types for new architecture
export interface SeatOccupant {
  playerId: string;
  username: string;
}

export interface Table {
  id: string;
  bundleId: string;
  owner: string;
  status: 'Open' | 'Countdown' | 'Playing' | 'Finished';
  seats: (SeatOccupant | null)[];
  readyStates: boolean[];
  minPlayers: number;
  maxPlayers: number;
  countdownEndsAt?: number; // Unix timestamp in seconds
  spectators?: string[];
}

export interface ChatMessage {
  id: string;
  scope: 'lobby' | 'table';
  tableId?: string;
  sender: string;
  message: string;
  timestamp: number; // Unix timestamp in seconds
}