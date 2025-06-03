// Game-specific type definitions

export interface GameEntity {
  id: string;
  props?: Record<string, any>;
  ui?: {
    glyph?: string;
    tokenType?: string;
    display?: string;
  };
}

export interface GameZone {
  id: string;
  type: 'grid' | 'list' | 'deck' | 'stack' | 'single';
  name?: string;
  cells?: any[][];
  items?: string[];
  cards?: string[];
  gridDimensions?: {
    rows: number;
    cols: number;
  };
  ui?: {
    layout?: string;
    checkerPattern?: boolean;
    rotateForPlayer?: boolean;
    showCount?: boolean;
    showTop?: boolean;
  };
  visibility?: string;
  group?: string;
  shape?: string;
}

export interface GameAction {
  action: string;
  direction?: string;
  args?: Record<string, any>;
}

export interface GameState {
  turn?: number;
  currentPlayer?: string;
  tick?: number;
  gameStatus?: {
    state: 'playing' | 'ended';
    winner?: string | null;
    tie?: boolean;
  };
  zones?: Record<string, any>;
  phases?: {
    currentPhase?: string;
    [key: string]: any;
  };
  selection?: {
    zone: string;
    row: number;
    col: number;
  };
}

export interface GameManifest {
  gameId: string;
  version: string;
  metadata: {
    name: string;
    description?: string;
    author?: string;
    players?: {
      min: number;
      max: number;
    };
  };
}