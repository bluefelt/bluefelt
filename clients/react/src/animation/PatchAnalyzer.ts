/**
 * Patch Analyzer for detecting animatable changes in JSON patches.
 * 
 * Analyzes incoming patches from the server to determine what animations
 * should be triggered based on the type and path of state changes.
 */

import {
  AnimationType,
  type AnimationPlan,
  type PatchOperation,
  type EntityMovementMetadata,
  type ZoneTransferMetadata,
  type TurnChangeMetadata,
  type CardDealMetadata,
  type CardFlipMetadata,
  type PieceCaptureMetadata,
  type ScoreChangeMetadata,
  ANIMATION_DURATIONS
} from './AnimationTypes';

export interface AnalysisContext {
  currentState: any;
  patch: PatchOperation;
  gameId?: string;
  currentPlayer?: string; // e.g., 'p1', 'p2'
}

export class PatchAnalyzer {
  
  /**
   * Analyze a patch for animation potential
   */
  analyzeForAnimation(context: AnalysisContext): AnimationPlan | null {
    const { patch, currentState } = context;
    
    // Skip analysis if patch path is invalid
    if (!patch.path || typeof patch.path !== 'string') {
      return null;
    }
    
    // Check for server-provided animation hints first
    if ((patch as any)._animation) {
      return this.createAnimationFromServerHint(patch, (patch as any)._animation, context);
    }
    
    // Fallback to client-side analysis for older patches without hints
    const detectors = [
      this.detectEntityMovement,
      this.detectEntitySpawn,
      this.detectEntityDestroy,
      this.detectZoneTransfer,
      this.detectSelectionChange,
      this.detectTurnChange,
      this.detectPhaseChange,
      this.detectGameEnd
    ];
    
    for (const detector of detectors) {
      const result = detector.call(this, context);
      if (result) {
        return result;
      }
    }
    
    return null;
  }
  
  /**
   * Detect entity movement between cells or zones
   */
  private detectEntityMovement(context: AnalysisContext): AnimationPlan | null {
    const { patch, currentState } = context;
    
    // Look for patches to zone cells that involve entity changes
    const cellMatch = patch.path.match(/^\/game\/zones\/([^\/]+)\/cells\/(\d+)\/(\d+)$/);
    if (!cellMatch) return null;
    
    const [, zoneId, row, col] = cellMatch;
    const rowNum = parseInt(row);
    const colNum = parseInt(col);
    
    // Check if this is a movement (old cell had entity, new cell gets it)
    const currentCell = currentState?.game?.zones?.[zoneId]?.cells?.[rowNum]?.[colNum];
    const newValue = patch.value;
    
    console.log('[PatchAnalyzer] Analyzing cell patch:', {
      path: patch.path,
      currentCell,
      newValue,
      hasEntity: newValue?.entity,
      currentPlayer: context.currentPlayer
    });
    
    // Entity being placed in empty cell
    if (!currentCell && newValue?.entity) {
      // Check if this is a gravity drop (Connect 4 style)
      // by looking for empty cells above this position
      const boardCells = currentState?.game?.zones?.[zoneId]?.cells;
      let isGravityDrop = false;
      let dropFromRow = 0;
      
      if (boardCells && Array.isArray(boardCells) && rowNum > 0) {
        // First check if there are any pieces in lower rows of the same column
        let hasLowerPieces = false;
        for (let checkRow = rowNum + 1; checkRow < boardCells.length; checkRow++) {
          if (boardCells[checkRow]?.[colNum]?.entity) {
            hasLowerPieces = true;
            break;
          }
        }
        
        // Only consider it a gravity drop if:
        // 1. There are pieces below (or it's the bottom row), AND
        // 2. There's at least one empty cell above
        if (hasLowerPieces || rowNum === boardCells.length - 1) {
          // Check all cells above this position in the same column
          for (let checkRow = 0; checkRow < rowNum; checkRow++) {
            if (!boardCells[checkRow]?.[colNum]?.entity) {
              isGravityDrop = true;
              dropFromRow = checkRow;
              break;
            }
          }
        }
      }
      
      if (import.meta.env.DEV && isGravityDrop) {
        console.log('[PatchAnalyzer] Gravity drop detected:', {
          entity: newValue.entity,
          fromRow: dropFromRow,
          toRow: rowNum,
          col: colNum
        });
      }
      
      // Use different animation types for gravity drops vs regular placements
      const isYourPiece = this.isCurrentPlayerEntity(newValue.entity, context.currentPlayer);
      
      if (isGravityDrop) {
        return this.createAnimationPlan({
          type: AnimationType.ENTITY_MOVEMENT,
          targetPath: patch.path,
          metadata: {
            toZone: zoneId,
            toPosition: { row: rowNum, col: colNum },
            fromPosition: { row: dropFromRow, col: colNum },
            entity: newValue.entity,
            isGravityDrop: true,
            isYourPiece
          } as EntityMovementMetadata
        });
      } else {
        // Regular placement (like Tic Tac Toe) - use spawn animation
        const isYourPiece = this.isCurrentPlayerEntity(newValue.entity, context.currentPlayer);
        return this.createAnimationPlan({
          type: AnimationType.ENTITY_SPAWN,
          targetPath: patch.path,
          metadata: {
            entity: newValue.entity,
            zone: zoneId,
            position: { row: rowNum, col: colNum },
            isYourPiece,
            gameId: context.gameId
          }
        });
      }
    }
    
    // Entity being removed from cell
    if (currentCell?.entity && (!newValue || !newValue.entity)) {
      return this.createAnimationPlan({
        type: AnimationType.ENTITY_DESTROY,
        targetPath: patch.path,
        metadata: {
          fromZone: zoneId,
          fromPosition: { row: rowNum, col: colNum },
          entity: currentCell.entity
        } as EntityMovementMetadata
      });
    }
    
    return null;
  }
  
  /**
   * Detect new entity spawning
   */
  private detectEntitySpawn(context: AnalysisContext): AnimationPlan | null {
    const { patch } = context;
    
    // Look for 'add' operations that create new entities
    if (patch.op === 'add' && patch.value?.entity) {
      return this.createAnimationPlan({
        type: AnimationType.ENTITY_SPAWN,
        targetPath: patch.path,
        metadata: {
          entity: patch.value.entity
        }
      });
    }
    
    return null;
  }
  
  /**
   * Detect entity destruction
   */
  private detectEntityDestroy(context: AnalysisContext): AnimationPlan | null {
    const { patch, currentState } = context;
    
    // Look for 'remove' operations or null replacements
    if (patch.op === 'remove' || (patch.op === 'replace' && patch.value === null)) {
      const currentValue = this.getValueAtPath(currentState, patch.path);
      
      if (currentValue?.entity) {
        return this.createAnimationPlan({
          type: AnimationType.ENTITY_DESTROY,
          targetPath: patch.path,
          metadata: {
            entity: currentValue.entity
          }
        });
      }
    }
    
    return null;
  }
  
  /**
   * Detect card/item transfers between zones
   */
  private detectZoneTransfer(context: AnalysisContext): AnimationPlan | null {
    const { patch } = context;
    
    // Look for changes to zone items (card games)
    const zoneItemsMatch = patch.path.match(/^\/game\/zones\/([^\/]+)\/items$/);
    if (!zoneItemsMatch) return null;
    
    const zoneId = zoneItemsMatch[1];
    
    // This could indicate cards being added/removed from a zone
    if (Array.isArray(patch.value)) {
      return this.createAnimationPlan({
        type: AnimationType.ZONE_TRANSFER,
        targetPath: patch.path,
        metadata: {
          toZone: zoneId,
          cardCount: patch.value.length,
          isVisible: !zoneId.includes('deck') // Assume deck cards are hidden
        } as ZoneTransferMetadata
      });
    }
    
    return null;
  }
  
  /**
   * Detect selection changes
   */
  private detectSelectionChange(context: AnalysisContext): AnimationPlan | null {
    const { patch } = context;
    
    if (patch.path.startsWith('/game/selection')) {
      return this.createAnimationPlan({
        type: AnimationType.SELECTION_CHANGE,
        targetPath: patch.path,
        metadata: {
          selectionData: patch.value
        }
      });
    }
    
    return null;
  }
  
  /**
   * Detect turn changes
   */
  private detectTurnChange(context: AnalysisContext): AnimationPlan | null {
    const { patch, currentState } = context;
    
    if (patch.path === '/game/currentPlayer') {
      const fromPlayer = currentState?.game?.currentPlayer;
      const toPlayer = patch.value;
      
      if (fromPlayer !== toPlayer) {
        return this.createAnimationPlan({
          type: AnimationType.TURN_CHANGE,
          targetPath: patch.path,
          metadata: {
            fromPlayer,
            toPlayer,
            turnNumber: currentState?.game?.turn || 0,
            isYourTurn: toPlayer === context.currentPlayer
          } as TurnChangeMetadata
        });
      }
    }
    
    return null;
  }
  
  /**
   * Detect phase changes
   */
  private detectPhaseChange(context: AnalysisContext): AnimationPlan | null {
    const { patch } = context;
    
    if (patch.path.startsWith('/game/phases/')) {
      return this.createAnimationPlan({
        type: AnimationType.PHASE_CHANGE,
        targetPath: patch.path,
        metadata: {
          phaseData: patch.value
        }
      });
    }
    
    return null;
  }
  
  /**
   * Detect game end conditions
   */
  private detectGameEnd(context: AnalysisContext): AnimationPlan | null {
    const { patch } = context;
    
    if (patch.path === '/game/gameStatus' || patch.path === '/ui/gameStatus') {
      const gameStatus = patch.value;
      
      if (gameStatus?.state === 'ended') {
        return this.createAnimationPlan({
          type: AnimationType.GAME_END,
          targetPath: patch.path,
          metadata: {
            winner: gameStatus.winner,
            tie: gameStatus.tie,
            isYou: gameStatus.winner === context.currentPlayer
          }
        });
      }
    }
    
    return null;
  }
  
  /**
   * Helper to create standardized animation plans
   */
  private createAnimationPlan(options: {
    type: AnimationType;
    targetPath: string;
    metadata?: Record<string, any>;
    duration?: number;
  }): AnimationPlan {
    const { type, targetPath, metadata = {}, duration: customDuration } = options;
    
    // Use custom duration if provided, otherwise get from defaults
    const duration = customDuration || ANIMATION_DURATIONS[type] || 400;
    
    return {
      id: `${type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type,
      duration,
      fromState: null, // Will be populated by animation engine
      toState: null,   // Will be populated by animation engine
      targetPath,
      metadata,
      priority: this.getAnimationPriority(type)
    };
  }
  
  /**
   * Get priority for different animation types
   */
  private getAnimationPriority(type: AnimationType): number {
    const priorities = {
      [AnimationType.GAME_END]: 1,             // Highest priority
      [AnimationType.SHAKE_ERROR]: 2,          // Error feedback should be immediate
      [AnimationType.TURN_CHANGE]: 3,
      [AnimationType.PHASE_CHANGE]: 4,
      [AnimationType.PIECE_CAPTURE]: 5,        // Important game actions
      [AnimationType.SCORE_CHANGE]: 6,
      [AnimationType.CARD_FLIP]: 7,           // Card reveals are important
      [AnimationType.ENTITY_MOVEMENT]: 8,
      [AnimationType.CARD_DEAL]: 9,
      [AnimationType.CARD_SHUFFLE]: 10,
      [AnimationType.CARD_COLLECT]: 11,
      [AnimationType.ZONE_TRANSFER]: 12,
      [AnimationType.ENTITY_SPAWN]: 13,
      [AnimationType.ENTITY_DESTROY]: 14,
      [AnimationType.MULTI_PIECE_MOVE]: 15,
      [AnimationType.HIGHLIGHT_ZONE]: 16,
      [AnimationType.SIMULTANEOUS_ACTION]: 17,
      [AnimationType.SELECTION_CHANGE]: 18    // Lowest priority
    };
    
    return priorities[type] || 20;
  }
  
  /**
   * Helper to get value at a JSON path
   */
  private getValueAtPath(obj: any, path: string): any {
    if (!obj || !path) return null;
    
    // Remove leading slash and split path
    const pathParts = path.replace(/^\//, '').split('/');
    
    let current = obj;
    for (const part of pathParts) {
      if (current === null || current === undefined) {
        return null;
      }
      current = current[part];
    }
    
    return current;
  }
  
  /**
   * Create animation plan from server-provided hint
   */
  private createAnimationFromServerHint(patch: PatchOperation, animationHint: any, context: AnalysisContext): AnimationPlan | null {
    if (!animationHint || !animationHint.type) {
      return null;
    }

    // Map server animation types to client animation types
    const typeMapping: Record<string, AnimationType> = {
      'entity_movement': AnimationType.ENTITY_MOVEMENT,
      'entity_spawn': AnimationType.ENTITY_SPAWN,
      'entity_remove': AnimationType.ENTITY_DESTROY,
      'zone_transfer': AnimationType.ZONE_TRANSFER,
      'turn_change': AnimationType.TURN_CHANGE,
      'phase_change': AnimationType.PHASE_CHANGE,
      'game_end': AnimationType.GAME_END,
      'card_deal': AnimationType.CARD_DEAL,
      'card_flip': AnimationType.CARD_FLIP,
      'card_shuffle': AnimationType.CARD_SHUFFLE,
      'card_collect': AnimationType.CARD_COLLECT,
      'piece_capture': AnimationType.PIECE_CAPTURE,
      'score_change': AnimationType.SCORE_CHANGE,
      'shake_error': AnimationType.SHAKE_ERROR
    };

    const animationType = typeMapping[animationHint.type];
    if (!animationType) {
      console.warn('[PatchAnalyzer] Unknown server animation type:', animationHint.type);
      return null;
    }

    // Use server-provided duration or fallback to defaults
    const duration = animationHint.duration || ANIMATION_DURATIONS[animationType] || 400;

    // Build metadata based on animation type and server hints
    let metadata: Record<string, any> = {};

    switch (animationType) {
      case AnimationType.ENTITY_MOVEMENT:
        metadata = {
          fromPath: animationHint.from,
          toPath: animationHint.to,
          isGravityDrop: animationHint.isGravityDrop || false,
          entity: patch.value?.entity
        } as EntityMovementMetadata;
        break;
        
      case AnimationType.ENTITY_SPAWN:
        // Check if this is actually a gravity drop
        if (animationHint.isGravityDrop) {
          // Convert to entity movement for gravity drops
          return this.createAnimationPlan({
            type: AnimationType.ENTITY_MOVEMENT,
            targetPath: patch.path,
            metadata: {
              toZone: 'board',
              toPosition: animationHint.toPosition || this.extractPositionFromPath(patch.path),
              fromPosition: animationHint.fromPosition || { row: 0, col: animationHint.toPosition?.col || 0 },
              entity: patch.value?.entity,
              isGravityDrop: true,
              isYourPiece: this.isCurrentPlayerEntity(patch.value?.entity, context.currentPlayer)
            } as EntityMovementMetadata,
            duration
          });
        }
        
        metadata = {
          entity: patch.value?.entity,
          spawnLocation: patch.path
        };
        break;
        
      case AnimationType.ENTITY_DESTROY:
        metadata = {
          entity: animationHint.entity,
          destroyLocation: patch.path
        };
        break;
        
      case AnimationType.ZONE_TRANSFER:
        metadata = {
          fromZone: animationHint.fromZone,
          toZone: animationHint.toZone,
          cardCount: animationHint.cardCount || 1
        } as ZoneTransferMetadata;
        break;
        
      case AnimationType.CARD_DEAL:
        metadata = {
          fromZone: animationHint.fromZone || 'deck',
          toZone: animationHint.toZone,
          cardCount: animationHint.cardCount || 1,
          dealerPlayer: animationHint.dealerPlayer,
          isVisible: animationHint.isVisible || false,
          dealSpeed: animationHint.dealSpeed || 'normal'
        };
        break;
        
      case AnimationType.CARD_FLIP:
        metadata = {
          cardId: animationHint.cardId,
          fromFaceDown: animationHint.fromFaceDown || true,
          zone: animationHint.zone,
          cardIndex: animationHint.cardIndex
        };
        break;
        
      case AnimationType.PIECE_CAPTURE:
        metadata = {
          capturedPiece: animationHint.capturedPiece,
          capturingPiece: animationHint.capturingPiece,
          fromPosition: animationHint.fromPosition,
          toPosition: animationHint.toPosition,
          captureMethod: animationHint.captureMethod || 'adjacent'
        };
        break;
        
      case AnimationType.SCORE_CHANGE:
        metadata = {
          playerId: animationHint.playerId,
          oldScore: animationHint.oldScore || 0,
          newScore: animationHint.newScore || 0,
          scoreDelta: animationHint.scoreDelta || 0,
          scoreType: animationHint.scoreType
        };
        break;
        
      default:
        // For other types, use hint data directly
        metadata = { ...animationHint };
        break;
    }

    if (import.meta.env.DEV) {
      console.log('[PatchAnalyzer] Created animation from server hint:', {
        type: animationType,
        duration,
        metadata,
        patch: patch.path
      });
    }

    return this.createAnimationPlan({
      type: animationType,
      targetPath: patch.path,
      metadata,
      duration
    });
  }

  /**
   * Check if two animation plans are similar enough to be deduplicated
   */
  canDeduplicateAnimation(plan1: AnimationPlan, plan2: AnimationPlan): boolean {
    return (
      plan1.type === plan2.type &&
      plan1.targetPath === plan2.targetPath &&
      Math.abs(plan1.priority - plan2.priority) <= 1
    );
  }
  
  /**
   * Sort animation plans by priority and timing
   */
  sortAnimationPlans(plans: AnimationPlan[]): AnimationPlan[] {
    return plans.sort((a, b) => {
      // First sort by priority (lower number = higher priority)
      if (a.priority !== b.priority) {
        return a.priority - b.priority;
      }
      
      // Then by creation time (embedded in ID)
      return a.id.localeCompare(b.id);
    });
  }
  
  /**
   * Determine if an entity belongs to the current player
   */
  private isCurrentPlayerEntity(entityId: string, currentPlayer?: string): boolean {
    if (!entityId || !currentPlayer) return false;
    
    // Check if entity ID contains player identifier
    // e.g., "x_p1", "mark_p2", "p1_piece", etc.
    return entityId.includes(currentPlayer);
  }
  
  /**
   * Extract row/col position from a path like "/game/zones/board/cells/0/3"
   */
  private extractPositionFromPath(path: string): { row: number; col: number } | null {
    const match = path.match(/\/cells\/(\d+)\/(\d+)$/);
    if (match) {
      return {
        row: parseInt(match[1]),
        col: parseInt(match[2])
      };
    }
    return null;
  }
}