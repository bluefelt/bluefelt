/**
 * Comprehensive tests for the Patch Analyzer
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { PatchAnalyzer, AnalysisContext } from '../animation/PatchAnalyzer';
import { 
  AnimationType, 
  type PatchOperation,
  type AnimationPlan,
  type EntityMovementMetadata,
  type ZoneTransferMetadata,
  type TurnChangeMetadata
} from '../animation/AnimationTypes';

describe('PatchAnalyzer', () => {
  let analyzer: PatchAnalyzer;

  beforeEach(() => {
    analyzer = new PatchAnalyzer();
  });

  describe('Animatable Patch Detection', () => {
    it('should detect entity spawn patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/0/0',
        value: { entity: 'x_p1' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [[null]]
              }
            }
          }
        },
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_SPAWN);
      expect(animationPlan?.metadata).toMatchObject({
        entity: 'x_p1',
        zone: 'board',
        position: { row: 0, col: 0 },
        isYourPiece: true
      });
    });

    it('should detect entity destruction patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/1/1',
        value: null
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [
                  [null, null],
                  [null, { entity: 'o_p2' }]
                ]
              }
            }
          }
        }
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_DESTROY);
      expect(animationPlan?.metadata).toMatchObject({
        fromZone: 'board',
        fromPosition: { row: 1, col: 1 },
        entity: 'o_p2'
      });
    });

    it('should detect zone transfer patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/hand_p1/items',
        value: ['card_1', 'card_2', 'card_3']
      };

      const context: AnalysisContext = {
        patch,
        currentState: {}
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ZONE_TRANSFER);
      expect(animationPlan?.metadata).toMatchObject({
        toZone: 'hand_p1',
        cardCount: 3,
        isVisible: true
      });
    });

    it('should detect turn change patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/currentPlayer',
        value: 'p2'
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            currentPlayer: 'p1',
            turn: 5
          }
        },
        currentPlayer: 'p2'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.TURN_CHANGE);
      
      const metadata = animationPlan?.metadata as TurnChangeMetadata;
      expect(metadata).toMatchObject({
        fromPlayer: 'p1',
        toPlayer: 'p2',
        turnNumber: 5,
        isYourTurn: true
      });
    });

    it('should detect game end patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/ui/gameStatus',
        value: { state: 'ended', winner: 'p1' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {},
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.GAME_END);
      expect(animationPlan?.metadata).toMatchObject({
        winner: 'p1',
        isYou: true
      });
    });

    it('should return null for non-animatable patches', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/someRandomProperty',
        value: 'test'
      };

      const context: AnalysisContext = {
        patch,
        currentState: {}
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeNull();
    });
  });

  describe('Gravity Drop Detection', () => {
    it('should detect gravity drops in Connect 4 style games', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/5/2',
        value: { entity: 'red_p1' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [
                  [null, null, null, null],
                  [null, null, null, null],
                  [null, null, null, null],
                  [null, null, null, null],
                  [null, null, { entity: 'yellow_p2' }, null],
                  [null, null, null, null]  // Changed: no piece at 5,2 yet
                ]
              }
            }
          }
        },
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_MOVEMENT);
      
      const metadata = animationPlan?.metadata as EntityMovementMetadata;
      expect(metadata.isGravityDrop).toBe(true);
      expect(metadata.fromPosition).toEqual({ row: 0, col: 2 });
      expect(metadata.toPosition).toEqual({ row: 5, col: 2 });
    });

    it('should not detect gravity drop for regular placement', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/1/1',
        value: { entity: 'x_p1' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [
                  [{ entity: 'o_p2' }, null, null],
                  [null, null, null],
                  [null, null, { entity: 'x_p1' }]
                ]
              }
            }
          }
        },
        currentPlayer: 'p1',
        gameId: 'tic-tac-toe'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_SPAWN);
      expect(animationPlan?.metadata).toHaveProperty('gameId', 'tic-tac-toe');
    });
  });

  describe('Animation Metadata Extraction', () => {
    it('should extract correct metadata for entity spawn', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/1/2',
        value: { entity: 'piece_p2' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [
                  [{ entity: 'x_p1' }, null, null],
                  [null, null, null], // Middle row - placing at 1,2
                  [null, { entity: 'o_p2' }, null]
                ]
              }
            }
          }
        },
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan?.metadata).toMatchObject({
        entity: 'piece_p2',
        zone: 'board',
        position: { row: 1, col: 2 },
        isYourPiece: false // p2 piece but current player is p1
      });
    });

    it('should identify player ownership correctly', () => {
      const testCases = [
        { entity: 'x_p1', currentPlayer: 'p1', expected: true },
        { entity: 'o_p2', currentPlayer: 'p1', expected: false },
        { entity: 'p1_piece', currentPlayer: 'p1', expected: true },
        { entity: 'mark_p2', currentPlayer: 'p2', expected: true },
        { entity: 'neutral', currentPlayer: 'p1', expected: false }
      ];

      for (const testCase of testCases) {
        const patch: PatchOperation = {
          op: 'replace',
          path: '/game/zones/board/cells/0/0',
          value: { entity: testCase.entity }
        };

        const context: AnalysisContext = {
          patch,
          currentState: { game: { zones: { board: { cells: [[null]] } } } },
          currentPlayer: testCase.currentPlayer
        };

        const animationPlan = analyzer.analyzeForAnimation(context);
        expect(animationPlan?.metadata.isYourPiece).toBe(testCase.expected);
      }
    });
  });

  describe('Different Patch Types', () => {
    it('should handle add operations', () => {
      const patch: PatchOperation = {
        op: 'add',
        path: '/game/zones/board/cells/0/0',
        value: { entity: 'new_piece' }
      };

      const context: AnalysisContext = {
        patch,
        currentState: {}
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_SPAWN);
    });

    it('should handle remove operations', () => {
      const patch: PatchOperation = {
        op: 'remove',
        path: '/game/zones/board/cells/0/0'
      };

      const context: AnalysisContext = {
        patch,
        currentState: {
          game: {
            zones: {
              board: {
                cells: [[{ entity: 'removed_piece' }]]
              }
            }
          }
        }
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_DESTROY);
    });

    it('should skip patches with invalid paths', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '',
        value: 'test'
      };

      const context: AnalysisContext = {
        patch,
        currentState: {}
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeNull();
    });
  });

  describe('Animation Priority and Sorting', () => {
    it('should assign correct priorities to different animation types', () => {
      const patches: Array<{ type: AnimationType; expectedPriority: number }> = [
        { type: AnimationType.GAME_END, expectedPriority: 1 },
        { type: AnimationType.SHAKE_ERROR, expectedPriority: 2 },
        { type: AnimationType.TURN_CHANGE, expectedPriority: 3 },
        { type: AnimationType.ENTITY_SPAWN, expectedPriority: 13 },
        { type: AnimationType.SELECTION_CHANGE, expectedPriority: 18 }
      ];

      for (const { type, expectedPriority } of patches) {
        const plan = (analyzer as any).createAnimationPlan({
          type,
          targetPath: '/test',
          metadata: {}
        });

        expect(plan.priority).toBe(expectedPriority);
      }
    });

    it('should sort animation plans by priority', () => {
      const plans: AnimationPlan[] = [
        (analyzer as any).createAnimationPlan({ type: AnimationType.ENTITY_SPAWN, targetPath: '/1' }),
        (analyzer as any).createAnimationPlan({ type: AnimationType.GAME_END, targetPath: '/2' }),
        (analyzer as any).createAnimationPlan({ type: AnimationType.TURN_CHANGE, targetPath: '/3' })
      ];

      const sorted = analyzer.sortAnimationPlans(plans);

      expect(sorted[0].type).toBe(AnimationType.GAME_END);
      expect(sorted[1].type).toBe(AnimationType.TURN_CHANGE);
      expect(sorted[2].type).toBe(AnimationType.ENTITY_SPAWN);
    });
  });

  describe('Animation Deduplication', () => {
    it('should identify duplicate animations', () => {
      const plan1 = (analyzer as any).createAnimationPlan({
        type: AnimationType.ENTITY_SPAWN,
        targetPath: '/game/zones/board/cells/0/0',
        metadata: { entity: 'x_p1' }
      });

      const plan2 = (analyzer as any).createAnimationPlan({
        type: AnimationType.ENTITY_SPAWN,
        targetPath: '/game/zones/board/cells/0/0',
        metadata: { entity: 'x_p1' }
      });

      const plan3 = (analyzer as any).createAnimationPlan({
        type: AnimationType.ENTITY_SPAWN,
        targetPath: '/game/zones/board/cells/1/1',
        metadata: { entity: 'o_p2' }
      });

      expect(analyzer.canDeduplicateAnimation(plan1, plan2)).toBe(true);
      expect(analyzer.canDeduplicateAnimation(plan1, plan3)).toBe(false);
    });
  });

  describe('Server Hint Processing', () => {
    it('should process server-provided animation hints', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/0/0',
        value: { entity: 'x_p1' },
        _animation: {
          type: 'entity_spawn',
          duration: 500
        }
      } as any;

      const context: AnalysisContext = {
        patch,
        currentState: {}
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_SPAWN);
      expect(animationPlan?.duration).toBe(500);
    });

    it('should handle gravity drop hints from server', () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/5/2',
        value: { entity: 'red_p1' },
        _animation: {
          type: 'entity_spawn',
          isGravityDrop: true,
          fromPosition: { row: 0, col: 2 },
          toPosition: { row: 5, col: 2 }
        }
      } as any;

      const context: AnalysisContext = {
        patch,
        currentState: {},
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_MOVEMENT);
      expect(animationPlan?.metadata).toMatchObject({
        isGravityDrop: true,
        fromPosition: { row: 0, col: 2 },
        toPosition: { row: 5, col: 2 }
      });
    });
  });
});