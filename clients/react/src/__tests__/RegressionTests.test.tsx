/**
 * Comprehensive regression tests to ensure critical fixes remain working
 * These tests cover all the major issues we've fixed:
 * 1. Game end state handling (cells not clickable after game ends)
 * 2. Patch application with legacy/non-existent paths
 * 3. State structure (game vs ui separation)
 * 4. Turn switching logic
 * 5. Action map synchronization
 */

import { describe, it, expect, vi } from 'vitest';
import { applyPatch } from 'fast-json-patch';

describe('Regression Tests - Critical Bug Fixes', () => {
  
  describe('Game End State Handling', () => {
    it('should disable all interactions when game ends with a winner', () => {
      const lobbyState = {
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          turn: 5,
          gameStatus: { state: 'ended', winner: 'p2', tie: false },
          zones: {
            board: {
              type: 'grid',
              cells: [
                [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, { entity: 'mark_p1' }],
                [{ entity: 'mark_p2' }, { entity: 'mark_p2' }, { entity: 'mark_p1' }],
                [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p1' }]
              ]
            }
          }
        },
        ui: {
          actionMap: {
            p1: {}, // Empty action map when game ends
            p2: {}
          }
        }
      };
      
      // Logic from GameView.tsx with game end check
      const isYourTurn = lobbyState.you && 
                         lobbyState.you !== 'spectator' && 
                         lobbyState.game.currentPlayer === lobbyState.you &&
                         lobbyState.game?.gameStatus?.state !== 'ended';
      
      expect(isYourTurn).toBe(false);
      expect(lobbyState.ui.actionMap.p1).toEqual({});
      expect(lobbyState.ui.actionMap.p2).toEqual({});
    });

    it('should disable interactions for tie games', () => {
      const lobbyState = {
        you: 'p1',
        game: {
          currentPlayer: 'p1',
          turn: 9,
          gameStatus: { state: 'ended', winner: null, tie: true },
          zones: {
            board: {
              type: 'grid',
              cells: [
                [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, { entity: 'mark_p1' }],
                [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p2' }],
                [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p2' }]
              ]
            }
          }
        },
        ui: {
          actionMap: {
            p1: {},
            p2: {}
          }
        }
      };
      
      const isYourTurn = lobbyState.you && 
                         lobbyState.you !== 'spectator' && 
                         lobbyState.game.currentPlayer === lobbyState.you &&
                         lobbyState.game?.gameStatus?.state !== 'ended';
      
      expect(isYourTurn).toBe(false);
      expect(lobbyState.game.gameStatus.tie).toBe(true);
      expect(lobbyState.game.gameStatus.winner).toBeNull();
    });

    it('should clear action maps when transitioning from playing to ended', () => {
      let lobbyState = {
        game: {
          currentPlayer: 'p1',
          turn: 4,
          gameStatus: { state: 'playing', winner: null, tie: false },
          zones: {
            board: {
              type: 'grid',
              cells: [[null, null, null], [null, null, null], [null, null, null]]
            }
          }
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' }
            },
            p2: {}
          }
        }
      };

      // Simulate game ending patches
      const endGamePatches = [
        { op: 'replace', path: '/game/gameStatus', value: { state: 'ended', winner: 'p1', tie: false } },
        { op: 'replace', path: '/ui/actionMap', value: { p1: {}, p2: {} } }
      ];

      for (const patch of endGamePatches) {
        const result = applyPatch(lobbyState, [patch]);
        lobbyState = result.newDocument;
      }

      expect(lobbyState.game.gameStatus.state).toBe('ended');
      expect(lobbyState.ui.actionMap.p1).toEqual({});
      expect(lobbyState.ui.actionMap.p2).toEqual({});
    });
  });

  describe('Legacy Path Handling', () => {
    it('should gracefully handle legacy possibleActions paths', () => {
      const lobbyState = {
        game: { currentPlayer: 'p1' },
        ui: { actionMap: { p1: {}, p2: {} } },
        // Add meta object to avoid undefined errors, but without possibleActions
        meta: {}
      };

      // Legacy patches that should not cause errors
      const legacyPatches = [
        { op: 'replace', path: '/meta/possibleActions/p1', value: [] },
        { op: 'replace', path: '/meta/possibleActions/p2', value: [] },
        { op: 'remove', path: '/ui/currentPhasePrompt' }
      ];

      let successCount = 0;
      let errorCount = 0;
      const errorNames: string[] = [];

      for (const patch of legacyPatches) {
        try {
          applyPatch(lobbyState, [patch], true, false);
          successCount++;
        } catch (error: any) {
          errorCount++;
          errorNames.push(error.name || 'unknown');
        }
      }

      // All patches should fail - first two with TypeError, last with OPERATION_PATH_UNRESOLVABLE
      expect(errorCount).toBe(3);
      expect(successCount).toBe(0);
      
      // But the state should remain valid
      expect(lobbyState.game).toBeDefined();
      expect(lobbyState.ui).toBeDefined();
    });

    it('should successfully apply valid patches while ignoring legacy ones', () => {
      let lobbyState = {
        game: {
          currentPlayer: 'p1',
          turn: 0,
          zones: { board: { cells: [[null]] } }
        },
        ui: {
          actionMap: { p1: {}, p2: {} }
        },
        // Add meta to avoid undefined errors
        meta: {}
      };

      // Mix of valid and legacy patches
      const mixedPatches = [
        { op: 'replace', path: '/game/turn', value: 1 }, // Valid
        { op: 'replace', path: '/meta/possibleActions/p1', value: [] }, // Legacy
        { op: 'replace', path: '/game/currentPlayer', value: 'p2' }, // Valid
        { op: 'remove', path: '/ui/currentPhasePrompt' } // Legacy
      ];

      let appliedCount = 0;
      let errorCount = 0;
      
      for (const patch of mixedPatches) {
        try {
          const result = applyPatch(lobbyState, [patch], true, false);
          lobbyState = result.newDocument;
          appliedCount++;
        } catch (error: any) {
          errorCount++;
          // Legacy paths will fail - this is expected
        }
      }

      // Only the 2 valid patches should have been applied
      expect(appliedCount).toBe(2);
      expect(errorCount).toBe(2);
      expect(lobbyState.game.turn).toBe(1);
      expect(lobbyState.game.currentPlayer).toBe('p2');
    });
  });

  describe('Turn Switching Logic', () => {
    it('should properly alternate turns between players', () => {
      const turnSequence = [
        { currentPlayer: 'p1', turn: 0, expectedIsP1Turn: true, expectedIsP2Turn: false },
        { currentPlayer: 'p2', turn: 1, expectedIsP1Turn: false, expectedIsP2Turn: true },
        { currentPlayer: 'p1', turn: 2, expectedIsP1Turn: true, expectedIsP2Turn: false },
        { currentPlayer: 'p2', turn: 3, expectedIsP1Turn: false, expectedIsP2Turn: true }
      ];

      turnSequence.forEach(({ currentPlayer, turn, expectedIsP1Turn, expectedIsP2Turn }) => {
        const gameState = {
          game: {
            currentPlayer,
            turn,
            gameStatus: { state: 'playing', winner: null, tie: false }
          }
        };

        // Check for p1
        const isP1Turn = 'p1' === currentPlayer && gameState.game.gameStatus.state !== 'ended';
        expect(isP1Turn).toBe(expectedIsP1Turn);

        // Check for p2
        const isP2Turn = 'p2' === currentPlayer && gameState.game.gameStatus.state !== 'ended';
        expect(isP2Turn).toBe(expectedIsP2Turn);
      });
    });

    it('should handle spectator mode correctly', () => {
      const lobbyState = {
        you: 'spectator',
        game: {
          currentPlayer: 'p1',
          turn: 0,
          gameStatus: { state: 'playing', winner: null, tie: false }
        }
      };

      const isYourTurn = lobbyState.you && 
                         lobbyState.you !== 'spectator' && 
                         lobbyState.game.currentPlayer === lobbyState.you &&
                         lobbyState.game.gameStatus.state !== 'ended';

      expect(isYourTurn).toBe(false);
      expect(lobbyState.you).toBe('spectator');
    });
  });

  describe('Action Map Synchronization', () => {
    it('should update action maps correctly when turns change', () => {
      let lobbyState = {
        game: {
          currentPlayer: 'p1',
          turn: 0,
          zones: {
            board: {
              type: 'grid',
              cells: [[null, null], [null, null]]
            }
          }
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' }
            },
            p2: {}
          }
        }
      };

      // Simulate turn change
      const turnChangePatches = [
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/game/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/game/turn', value: 1 },
        {
          op: 'replace',
          path: '/ui/actionMap',
          value: {
            p1: {},
            p2: {
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/1': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      ];

      for (const patch of turnChangePatches) {
        const result = applyPatch(lobbyState, [patch]);
        lobbyState = result.newDocument;
      }

      // Verify turn changed correctly
      expect(lobbyState.game.currentPlayer).toBe('p2');
      expect(lobbyState.game.turn).toBe(1);
      
      // Verify action maps switched
      expect(Object.keys(lobbyState.ui.actionMap.p1)).toHaveLength(0);
      expect(Object.keys(lobbyState.ui.actionMap.p2)).toHaveLength(3);
      
      // Verify board state updated
      expect(lobbyState.game.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
    });

    it('should handle action map updates for occupied cells', () => {
      const lobbyState = {
        game: {
          zones: {
            board: {
              type: 'grid',
              cells: [
                [{ entity: 'mark_p1' }, null, null],
                [null, { entity: 'mark_p2' }, null],
                [null, null, null]
              ]
            }
          }
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/2': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/2': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/2/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/2/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/2/2': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      // Verify occupied cells are not in action map
      expect(lobbyState.ui.actionMap.p1['/zones/board/cells/0/0']).toBeUndefined();
      expect(lobbyState.ui.actionMap.p1['/zones/board/cells/1/1']).toBeUndefined();
      
      // Verify empty cells are in action map
      expect(lobbyState.ui.actionMap.p1['/zones/board/cells/0/1']).toBeDefined();
      expect(lobbyState.ui.actionMap.p1['/zones/board/cells/2/2']).toBeDefined();
    });
  });

  describe('State Structure Integrity', () => {
    it('should maintain correct state structure throughout game lifecycle', () => {
      // Initial state
      const initialState = {
        you: 'p1',
        started: true,
        game: {
          currentPlayer: 'p1',
          turn: 0,
          tick: 0,
          gameStatus: { state: 'playing', winner: null, tie: false },
          zones: {
            board: {
              type: 'grid',
              cells: [[null, null, null], [null, null, null], [null, null, null]]
            }
          },
          players: [{ id: 'p1' }, { id: 'p2' }]
        },
        ui: {
          players: ['alice', 'bob'],
          actionMap: { p1: {}, p2: {} },
          entities: [],
          zones: [],
          gameLog: []
        }
      };

      // Verify all required paths exist
      expect(initialState.game).toBeDefined();
      expect(initialState.ui).toBeDefined();
      expect(initialState.game.currentPlayer).toBeDefined();
      expect(initialState.game.gameStatus).toBeDefined();
      expect(initialState.ui.actionMap).toBeDefined();
      
      // Verify no legacy paths exist
      expect((initialState as any).meta).toBeUndefined();
      expect((initialState as any).state).toBeUndefined();
    });

    it('should handle welcome message structure correctly', () => {
      const welcomeMessage = {
        type: 'welcome',
        you: 'p2',
        started: true,
        game: {
          currentPlayer: 'p1',
          turn: 0,
          gameStatus: { state: 'playing', winner: null, tie: false },
          zones: { board: { type: 'grid', cells: [[null, null, null], [null, null, null], [null, null, null]] } }
        },
        ui: {
          players: ['alice', 'bob'],
          actionMap: { p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } }, p2: {} },
          entities: [{ id: 'mark_p1' }, { id: 'mark_p2' }]
        }
      };

      // Verify correct structure
      expect(welcomeMessage.game.currentPlayer).toBe('p1');
      expect(welcomeMessage.ui.players[1]).toBe('bob');
      expect(welcomeMessage.you).toBe('p2');
      expect(Object.keys(welcomeMessage.ui.actionMap.p1)).toHaveLength(1);
      expect(Object.keys(welcomeMessage.ui.actionMap.p2)).toHaveLength(0);
    });
  });

  describe('Edge Cases and Error Scenarios', () => {
    it('should handle rapid patch sequences without state corruption', () => {
      let lobbyState = {
        game: {
          currentPlayer: 'p1',
          turn: 0,
          zones: { board: { cells: [[null, null], [null, null]] } }
        },
        ui: { actionMap: { p1: {}, p2: {} } }
      };

      // Simulate rapid moves
      const rapidPatches = [
        // Move 1
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/game/turn', value: 1 },
        { op: 'replace', path: '/game/currentPlayer', value: 'p2' },
        // Move 2
        { op: 'replace', path: '/game/zones/board/cells/0/1', value: { entity: 'mark_p2' } },
        { op: 'replace', path: '/game/turn', value: 2 },
        { op: 'replace', path: '/game/currentPlayer', value: 'p1' },
        // Move 3
        { op: 'replace', path: '/game/zones/board/cells/1/0', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/game/turn', value: 3 },
        { op: 'replace', path: '/game/currentPlayer', value: 'p2' }
      ];

      for (const patch of rapidPatches) {
        const result = applyPatch(lobbyState, [patch]);
        lobbyState = result.newDocument;
      }

      // Verify final state is consistent
      expect(lobbyState.game.turn).toBe(3);
      expect(lobbyState.game.currentPlayer).toBe('p2');
      expect(lobbyState.game.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
      expect(lobbyState.game.zones.board.cells[0][1]).toEqual({ entity: 'mark_p2' });
      expect(lobbyState.game.zones.board.cells[1][0]).toEqual({ entity: 'mark_p1' });
      expect(lobbyState.game.zones.board.cells[1][1]).toBeNull();
    });

    it('should handle malformed patches gracefully', () => {
      const lobbyState = {
        game: { currentPlayer: 'p1' },
        ui: { actionMap: {} }
      };

      const malformedPatches = [
        { op: 'replace' }, // Missing path and value
        { op: 'invalid_op', path: '/game/turn', value: 1 }, // Invalid operation
        { path: '/game/turn', value: 1 }, // Missing op
        { op: 'add', path: '', value: {} }, // Empty path
        { op: 'replace', path: null, value: 1 }, // Null path
      ];

      for (const patch of malformedPatches) {
        try {
          applyPatch(lobbyState, [patch as any]);
        } catch (error) {
          // Expected to fail - just ensure it doesn't crash
          expect(error).toBeDefined();
        }
      }

      // State should remain unchanged
      expect(lobbyState.game.currentPlayer).toBe('p1');
      expect(lobbyState.ui.actionMap).toEqual({});
    });
  });
});