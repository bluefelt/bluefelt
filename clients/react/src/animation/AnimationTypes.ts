/**
 * Core animation types and interfaces for the Bluefelt animation system.
 * 
 * This system intercepts JSON patches and creates visual animations based on 
 * state changes while maintaining strict separation between game logic and presentation.
 */

export enum AnimationType {
  ENTITY_MOVEMENT = 'entity_movement',     // Pieces moving between zones/cells
  ENTITY_SPAWN = 'entity_spawn',           // New pieces appearing
  ENTITY_DESTROY = 'entity_destroy',       // Pieces disappearing  
  ZONE_TRANSFER = 'zone_transfer',         // Cards between hands/deck
  SELECTION_CHANGE = 'selection_change',    // UI selection updates
  TURN_CHANGE = 'turn_change',             // Current player transitions
  PHASE_CHANGE = 'phase_change',           // Game phase transitions
  GAME_END = 'game_end',                   // Victory/defeat animations
  
  // Enhanced card game animations
  CARD_DEAL = 'card_deal',                 // Dealing cards from deck
  CARD_FLIP = 'card_flip',                 // Revealing hidden cards
  CARD_SHUFFLE = 'card_shuffle',           // Deck shuffling animation
  CARD_COLLECT = 'card_collect',           // Collecting cards (e.g., tricks)
  
  // Strategic game animations
  PIECE_CAPTURE = 'piece_capture',         // Capturing opponent pieces
  MULTI_PIECE_MOVE = 'multi_piece_move',   // Moving multiple pieces at once
  
  // UI feedback animations
  SCORE_CHANGE = 'score_change',           // Score updates
  HIGHLIGHT_ZONE = 'highlight_zone',       // Highlighting valid move zones
  SHAKE_ERROR = 'shake_error',             // Invalid move feedback
  
  // Advanced interactions
  SIMULTANEOUS_ACTION = 'simultaneous_action' // Multiple players acting at once
}

export interface AnimationConfig {
  speed: number;                    // 0.5x to 3x speed multiplier
  enableAnimations: boolean;        // Global toggle
  stillnessBetween: number;         // Delay between queued animations (ms)
  reduceMotion: boolean;           // Accessibility preference
  audioEnabled: boolean;           // Sound effects toggle
  audioVolume: number;             // 0-1 volume level
  enableTurnNotifications: boolean; // Play sound when it's your turn
  enableActionSounds: boolean;      // Play sounds for game actions
  maxQueueSize: number;            // Prevent memory issues
}

export interface AnimationPlan {
  id: string;                      // Unique identifier
  type: AnimationType;
  duration: number;                 // Base duration before speed adjustment
  fromState: any;                   // Visual state before
  toState: any;                     // Visual state after
  targetPath: string;               // JSON path being animated
  metadata: Record<string, any>;    // Animation-specific data
  priority: number;                 // Higher priority animations go first
}

export interface AnimationResult {
  success: boolean;
  animationId: string;
  actualDuration: number;
  error?: string;
}

export interface PatchOperation {
  op: 'add' | 'remove' | 'replace' | 'move' | 'copy' | 'test';
  path: string;
  value?: any;
  from?: string;
}

export interface EntityMovementMetadata {
  fromZone: string;
  toZone: string;
  fromPosition?: { row: number; col: number };
  toPosition?: { row: number; col: number };
  entity: string;
  isGravityDrop?: boolean;
}

export interface ZoneTransferMetadata {
  fromZone: string;
  toZone: string;
  cardCount: number;
  isVisible: boolean;
}

export interface TurnChangeMetadata {
  fromPlayer: string;
  toPlayer: string;
  turnNumber: number;
  isYourTurn?: boolean;
}

export interface CardDealMetadata {
  fromZone: string;
  toZone: string;
  cardCount: number;
  dealerPlayer: string;
  isVisible: boolean;
  dealSpeed?: 'slow' | 'normal' | 'fast';
}

export interface CardFlipMetadata {
  cardId: string;
  fromFaceDown: boolean;
  zone: string;
  cardIndex?: number;
}

export interface CardShuffleMetadata {
  zone: string;
  cardCount: number;
  shuffleIntensity: 'light' | 'normal' | 'heavy';
}

export interface PieceCaptureMetadata {
  capturedPiece: string;
  capturingPiece: string;
  fromPosition: { row: number; col: number };
  toPosition: { row: number; col: number };
  captureMethod: 'adjacent' | 'jump' | 'surround';
}

export interface ScoreChangeMetadata {
  playerId: string;
  oldScore: number;
  newScore: number;
  scoreDelta: number;
  scoreType?: string; // e.g., 'points', 'tricks', 'coins'
}

export interface ErrorFeedbackMetadata {
  errorType: 'invalid_move' | 'not_your_turn' | 'game_rule_violation';
  targetElement?: string;
  message?: string;
}

export interface AnimationState {
  isAnimating: boolean;
  currentAnimations: Set<string>;
  queuedAnimations: AnimationPlan[];
  config: AnimationConfig;
  // Compatibility property for tests
  queue?: AnimationPlan[];
}

/**
 * Default animation configuration with sensible defaults
 */
export const DEFAULT_ANIMATION_CONFIG: AnimationConfig = {
  speed: 1.0,
  enableAnimations: true,
  stillnessBetween: 150,
  reduceMotion: false,
  audioEnabled: true,
  audioVolume: 0.5,
  enableTurnNotifications: true,
  enableActionSounds: true,
  maxQueueSize: 20
};

/**
 * Animation duration constants in milliseconds
 */
export const ANIMATION_DURATIONS = {
  [AnimationType.ENTITY_MOVEMENT]: 400,
  [AnimationType.ENTITY_SPAWN]: 300,
  [AnimationType.ENTITY_DESTROY]: 250,
  [AnimationType.ZONE_TRANSFER]: 500,
  [AnimationType.SELECTION_CHANGE]: 150,
  [AnimationType.TURN_CHANGE]: 200,
  [AnimationType.PHASE_CHANGE]: 300,
  [AnimationType.GAME_END]: 800,
  
  // Enhanced animations
  [AnimationType.CARD_DEAL]: 350,
  [AnimationType.CARD_FLIP]: 400,
  [AnimationType.CARD_SHUFFLE]: 800,
  [AnimationType.CARD_COLLECT]: 600,
  [AnimationType.PIECE_CAPTURE]: 500,
  [AnimationType.MULTI_PIECE_MOVE]: 600,
  [AnimationType.SCORE_CHANGE]: 300,
  [AnimationType.HIGHLIGHT_ZONE]: 200,
  [AnimationType.SHAKE_ERROR]: 250,
  [AnimationType.SIMULTANEOUS_ACTION]: 450
} as const;

/**
 * CSS classes for different animation states
 */
export const ANIMATION_CLASSES = {
  MOVING: 'bf-animating-movement',
  SPAWNING: 'bf-animating-spawn',
  DESTROYING: 'bf-animating-destroy',
  TRANSFERRING: 'bf-animating-transfer',
  SELECTING: 'bf-animating-selection',
  TURN_HIGHLIGHT: 'bf-animating-turn',
  GAME_END_CELEBRATION: 'bf-animating-victory',
  
  // Enhanced classes
  CARD_DEALING: 'bf-animating-deal',
  CARD_FLIPPING: 'bf-animating-flip',
  CARD_SHUFFLING: 'bf-animating-shuffle',
  CARD_COLLECTING: 'bf-animating-collect',
  PIECE_CAPTURING: 'bf-animating-capture',
  MULTI_MOVING: 'bf-animating-multi-move',
  SCORE_UPDATING: 'bf-animating-score',
  ZONE_HIGHLIGHTING: 'bf-animating-highlight',
  ERROR_SHAKING: 'bf-animating-error',
  SIMULTANEOUS: 'bf-animating-simultaneous'
} as const;

/**
 * Audio event types for sound effects
 */
export enum AudioEventType {
  PIECE_MOVE = 'piece_move',
  PIECE_PLACE = 'piece_place',
  CARD_DRAW = 'card_draw',
  CARD_PLAY = 'card_play',
  TURN_CHANGE = 'turn_change',
  GAME_WIN = 'game_win',
  GAME_LOSE = 'game_lose',
  SELECTION = 'selection',
  ERROR = 'error',
  
  // Enhanced audio events
  CARD_FLIP = 'card_flip',
  CARD_SHUFFLE = 'card_shuffle',
  CARD_COLLECT = 'card_collect',
  PIECE_CAPTURE = 'piece_capture',
  SCORE_CHANGE = 'score_change',
  ZONE_HIGHLIGHT = 'zone_highlight',
  MULTI_ACTION = 'multi_action'
}

export interface AudioEvent {
  type: AudioEventType;
  volume?: number;
  pitch?: number;
}