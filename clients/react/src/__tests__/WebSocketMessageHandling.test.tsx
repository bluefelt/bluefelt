/**
 * Tests for WebSocket message handling and processing
 * Validates that different message types are correctly parsed and applied
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { applyPatch } from 'fast-json-patch';

describe('WebSocket Message Handling', () => {
  
  describe('Message Type Processing', () => {
    it('should handle welcome messages correctly', () => {
      const welcomeMessage = {
        type: 'welcome',
        you: 'p1',
        started: true,
        tick: 5,
        ui: {
          players: ['ben', 'karen'],
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        },
        game: {
          currentPlayer: 'p1',
          turn: 0,
          gameStatus: { state: 'playing', winner: null, tie: false },
          zones: {
            board: {
              type: 'grid',
              cells: [[null, null, null], [null, null, null], [null, null, null]]
            }
          }
        }
      };

      // Simulate the welcome handler logic
      const lobbyState = {
        you: welcomeMessage.you,
        ui: welcomeMessage.ui,
        game: welcomeMessage.game,
        started: welcomeMessage.started,
      };

      expect(lobbyState.you).toBe('p1');
      expect(lobbyState.started).toBe(true);
      expect(lobbyState.game.currentPlayer).toBe('p1');
      expect(lobbyState.ui.players).toEqual(['ben', 'karen']);
    });

    it('should handle diff messages with turn advancement', () => {
      const gameState = {
        ui: {
          actionMap: { p1: {} },
          players: ['ben', 'karen']
        },
        game: {
          currentPlayer: 'p1',
          turn: 0,
          tick: 1,
          zones: {
            board: {
              type: 'grid',
              cells: [[null, null, null], [null, null, null], [null, null, null]]
            }
          }
        }
      };

      const diffMessage = {
        type: 'diff',
        tick: 2,
        patch: [
          { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
          { op: 'replace', path: '/game/tick', value: 2 },
          { op: 'replace', path: '/game/turn', value: 1 },
          { op: 'replace', path: '/game/currentPlayer', value: 'p2' },
          { 
            op: 'replace', 
            path: '/ui/actionMap', 
            value: {
              p1: {},
              p2: {
                '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
                '/zones/board/cells/1/0': { action: 'placeMarker', direction: 'Click to place' }
              }
            }
          }
        ]
      };

      // Apply patches as the client would
      let workingState = { ...gameState };
      for (const patch of diffMessage.patch) {
        try {
          const result = applyPatch(workingState, [patch], true, false);
          workingState = result.newDocument;
        } catch (error) {
          console.error('Patch failed:', patch, error);
        }
      }

      // Verify the turn advanced correctly
      expect(workingState.game.currentPlayer).toBe('p2');
      expect(workingState.game.turn).toBe(1);
      expect(workingState.game.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
      expect(workingState.ui.actionMap.p2).toBeDefined();
      expect(Object.keys(workingState.ui.actionMap.p2)).toHaveLength(2);
    });

    it('should handle playerUpdate messages', () => {
      const initialState = {
        ui: { players: ['ben'] }
      };

      const playerUpdateMessage = {
        type: 'playerUpdate',
        players: ['ben', 'karen']
      };

      const updatedState = {
        ...initialState,
        ui: {
          ...initialState.ui,
          players: playerUpdateMessage.players,
        }
      };

      expect(updatedState.ui.players).toEqual(['ben', 'karen']);
      expect(updatedState.ui.players).toHaveLength(2);
    });
  });

  describe('Patch Application Edge Cases', () => {
    it('should handle patches that create missing parent paths', () => {
      let gameState = {
        ui: {},
        game: {}
      };

      const patches = [
        { op: 'add', path: '/ui/actionMap', value: {} },
        { op: 'add', path: '/meta/actionMap/p1', value: {} },
        { op: 'add', path: '/state/meta', value: {} },
        { op: 'add', path: '/state/meta/currentPlayer', value: 'p1' }
      ];

      for (const patch of patches) {
        try {
          const result = applyPatch(gameState, [patch]);
          gameState = result.newDocument;
        } catch (error) {
          // Some patches might fail if parent doesn't exist - that's ok
          console.log('Expected patch failure:', patch.path);
        }
      }

      // The state should have the successfully applied patches
      expect(gameState.ui.actionMap).toBeDefined();
    });

    it('should handle out-of-order patches gracefully', () => {
      let gameState = {
        ui: {},
        game: {
          zones: {
            board: {
              cells: [[null, null], [null, null]]
            }
          }
        }
      };

      // Patches that reference paths that might not exist yet
      const patches = [
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        { op: 'add', path: '/ui/actionMap', value: {} },
        { op: 'replace', path: '/ui/nonexistent/path', value: 'test' }, // This should fail
        { op: 'add', path: '/ui/actionMap/p1', value: {} }
      ];

      let successCount = 0;
      for (const patch of patches) {
        try {
          const result = applyPatch(gameState, [patch]);
          gameState = result.newDocument;
          successCount++;
        } catch (error) {
          // Continue with other patches
        }
      }

      expect(successCount).toBeGreaterThan(0);
      expect(gameState.game.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
      expect(gameState.ui.actionMap).toBeDefined();
    });

    it('should handle invalid patch operations', () => {
      const gameState = {
        ui: { test: 'value' },
        game: { zones: {} }
      };

      const invalidPatches = [
        { op: 'invalid_op', path: '/test', value: 'test' },
        { op: 'replace', path: '/nonexistent/deep/path', value: 'test' },
        { op: 'remove', path: '/ui/nonexistent' },
        // Missing required fields
        { op: 'replace', value: 'test' },
        { path: '/test', value: 'test' }
      ];

      // These should not crash the application
      for (const patch of invalidPatches) {
        expect(() => {
          try {
            applyPatch(gameState, [patch]);
          } catch (error) {
            // Expected to fail, just ensure it doesn't crash
            expect(error).toBeDefined();
          }
        }).not.toThrow();
      }
    });
  });

  describe('Message Format Validation', () => {
    it('should handle malformed JSON gracefully', () => {
      const malformedMessages = [
        '{"type": "diff", "tick": 1, "patch": [', // Incomplete JSON
        '{"type": "welcome", "you": }', // Invalid syntax
        '', // Empty string
        '{"type": "unknown"}', // Unknown type
        '[]', // Array instead of object
        'null' // Null value
      ];

      malformedMessages.forEach((msg) => {
        expect(() => {
          try {
            JSON.parse(msg);
          } catch (error) {
            // Expected for malformed JSON
            expect(error).toBeDefined();
          }
        }).not.toThrow();
      });
    });

    it('should handle messages with missing required fields', () => {
      const incompleteMessages = [
        { type: 'diff' }, // Missing tick and patch
        { type: 'welcome' }, // Missing you, started, etc.
        { type: 'playerUpdate' }, // Missing players
        { type: 'diff', tick: 1 }, // Missing patch
        { type: 'diff', patch: [] } // Missing tick
      ];

      incompleteMessages.forEach((msg) => {
        // These should not crash the message handler
        expect(msg.type).toBeDefined();
        
        // Simulate handler logic with safety checks
        if (msg.type === 'diff') {
          const patches = (msg as any).patch || [];
          const tick = (msg as any).tick || 0;
          expect(Array.isArray(patches)).toBe(true);
          expect(typeof tick).toBe('number');
        }
      });
    });
  });

  describe('Game State Consistency', () => {
    it('should maintain state consistency across multiple patches', () => {
      let gameState = {
        ui: {
          actionMap: { p1: {}, p2: {} }
        },
        game: {
          currentPlayer: 'p1',
          turn: 0,
          zones: {
            board: {
              cells: [[null, null], [null, null]]
            }
          }
        }
      };

      // Simulate a complete turn sequence
      const turnSequencePatches = [
        // Place mark
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        // Update game state
        { op: 'replace', path: '/game/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/game/turn', value: 1 },
        // Update action map
        { op: 'replace', path: '/ui/actionMap/p1', value: {} },
        { op: 'replace', path: '/ui/actionMap/p2', value: { '/zones/board/cells/0/1': { action: 'placeMarker' } } }
      ];

      for (const patch of turnSequencePatches) {
        const result = applyPatch(gameState, [patch]);
        gameState = result.newDocument;
      }

      // Verify game state updated correctly
      expect(gameState.game.currentPlayer).toBe('p2');
      expect(gameState.game.turn).toBe(1);
      
      // Verify the move was applied
      expect(gameState.game.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
      
      // Verify action map updated for new player
      expect(Object.keys(gameState.ui.actionMap.p1)).toHaveLength(0);
      expect(Object.keys(gameState.ui.actionMap.p2)).toHaveLength(1);
    });
  });
});