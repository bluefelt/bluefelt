/**
 * Integration tests for complete tic-tac-toe game scenarios
 * Tests full game flows and complex state transitions
 */

import { describe, it, expect, vi } from 'vitest';
import { applyPatch } from 'fast-json-patch';

describe('Tic-Tac-Toe Integration Tests', () => {

  describe('Complete Game Flow', () => {
    it('should handle a complete game from start to win', () => {
      // Initial game state
      let gameState = {
        you: 'p1',
        meta: {
          players: ['ben', 'karen'],
          entities: [
            { id: 'mark_p1', props: { value: 'p1' }, ui: { tokenType: 'p1' } },
            { id: 'mark_p2', props: { value: 'p2' }, ui: { tokenType: 'p2' } }
          ],
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/2': { action: 'placeMarker', direction: 'Click to place' }
            },
            p2: {}
          }
        },
        state: {
          meta: {
            currentPlayer: 'p1',
            turn: 0,
            tick: 1,
            gameStatus: { state: 'playing', winner: null, tie: false }
          },
          zones: {
            board: {
              type: 'grid',
              cells: [
                [null, null, null],
                [null, null, null],
                [null, null, null]
              ]
            }
          }
        },
        started: true
      };

      // Simulate game progression
      const gameSequence = [
        {
          move: 'p1 places at (0,0)',
          patches: [
            { op: 'replace', path: '/state/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
            { op: 'replace', path: '/meta/tick', value: 2 },
            { op: 'replace', path: '/meta/turn', value: 1 },
            { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
            { op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' },
            { op: 'replace', path: '/state/meta/turn', value: 1 },
            { 
              op: 'replace', 
              path: '/meta/actionMap/p1', 
              value: {} 
            },
            { 
              op: 'replace', 
              path: '/meta/actionMap/p2', 
              value: {
                '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
                '/zones/board/cells/0/2': { action: 'placeMarker', direction: 'Click to place' },
                '/zones/board/cells/1/0': { action: 'placeMarker', direction: 'Click to place' }
              }
            }
          ],
          expectedState: {
            currentPlayer: 'p2',
            turn: 1,
            boardState: { 0: { 0: { entity: 'mark_p1' } } }
          }
        },
        {
          move: 'p2 places at (1,0)',
          patches: [
            { op: 'replace', path: '/state/zones/board/cells/1/0', value: { entity: 'mark_p2' } },
            { op: 'replace', path: '/meta/tick', value: 3 },
            { op: 'replace', path: '/meta/turn', value: 0 },
            { op: 'replace', path: '/meta/currentPlayer', value: 'p1' },
            { op: 'replace', path: '/state/meta/currentPlayer', value: 'p1' },
            { op: 'replace', path: '/state/meta/turn', value: 0 },
            { 
              op: 'replace', 
              path: '/meta/actionMap/p2', 
              value: {} 
            },
            { 
              op: 'replace', 
              path: '/meta/actionMap/p1', 
              value: {
                '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
                '/zones/board/cells/0/2': { action: 'placeMarker', direction: 'Click to place' }
              }
            }
          ]
        },
        {
          move: 'p1 places at (0,1) - winning move',
          patches: [
            { op: 'replace', path: '/state/zones/board/cells/0/1', value: { entity: 'mark_p1' } },
            { op: 'replace', path: '/meta/tick', value: 4 },
            { op: 'replace', path: '/meta/turn', value: 1 },
            { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
            { op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' },
            { op: 'replace', path: '/state/meta/turn', value: 1 },
            { op: 'replace', path: '/state/meta/gameStatus', value: { state: 'ended', winner: 'p1', tie: false } },
            { op: 'replace', path: '/meta/actionMap/p1', value: {} },
            { op: 'replace', path: '/meta/actionMap/p2', value: {} }
          ]
        }
      ];

      // Apply each move and verify state
      gameSequence.forEach((sequence, index) => {
        console.log(`Applying ${sequence.move}`);
        
        for (const patch of sequence.patches) {
          try {
            const result = applyPatch(gameState, [patch]);
            gameState = result.newDocument;
          } catch (error) {
            console.error('Patch failed:', patch, error);
          }
        }

        if (sequence.expectedState) {
          expect(gameState.state.meta.currentPlayer).toBe(sequence.expectedState.currentPlayer);
          expect(gameState.state.meta.turn).toBe(sequence.expectedState.turn);
        }

        // Verify UI state
        const isYourTurn = gameState.you === gameState.state.meta.currentPlayer;
        const playerActions = gameState.meta?.actionMap?.[gameState.you] || {};
        const gameStatus = gameState.state.meta.gameStatus;

        if (index < gameSequence.length - 1) { // Not the final move
          if (isYourTurn) {
            expect(Object.keys(playerActions).length).toBeGreaterThan(0);
          }
        } else { // Final move - game should be ended
          expect(gameStatus.state).toBe('ended');
          expect(gameStatus.winner).toBe('p1');
          expect(Object.keys(playerActions)).toHaveLength(0); // No more moves
        }
      });
    });

    it('should handle a tie game correctly', () => {
      let gameState = {
        you: 'p1',
        state: {
          meta: {
            currentPlayer: 'p1',
            turn: 8, // Final move
            gameStatus: { state: 'playing', winner: null, tie: false }
          },
          zones: {
            board: {
              cells: [
                [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, { entity: 'mark_p1' }],
                [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p2' }],
                [{ entity: 'mark_p2' }, null, { entity: 'mark_p2' }] // Last move at (2,1)
              ]
            }
          }
        },
        meta: {
          actionMap: {
            p1: {
              '/zones/board/cells/2/1': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      // Final move that results in tie
      const finalMovePatches = [
        { op: 'replace', path: '/state/zones/board/cells/2/1', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/state/meta/gameStatus', value: { state: 'ended', winner: null, tie: true } },
        { op: 'replace', path: '/meta/actionMap/p1', value: {} },
        { op: 'replace', path: '/meta/actionMap/p2', value: {} }
      ];

      for (const patch of finalMovePatches) {
        const result = applyPatch(gameState, [patch]);
        gameState = result.newDocument;
      }

      const gameStatus = gameState.state.meta.gameStatus;
      expect(gameStatus.state).toBe('ended');
      expect(gameStatus.winner).toBeNull();
      expect(gameStatus.tie).toBe(true);

      // Verify board is full
      const board = gameState.state.zones.board.cells;
      const isFull = board.every((row: any[]) => 
        row.every((cell: any) => cell !== null && cell.entity)
      );
      expect(isFull).toBe(true);
    });
  });

  describe('Connection and Reconnection Scenarios', () => {
    it('should handle mid-game reconnection correctly', () => {
      // Simulate receiving welcome message after reconnection
      const midGameWelcome = {
        type: 'welcome',
        you: 'p2',
        started: true,
        tick: 5,
        meta: {
          players: ['ben', 'karen'],
          actionMap: {
            p1: {},
            p2: {
              '/zones/board/cells/1/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/2/2': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        },
        state: {
          meta: {
            currentPlayer: 'p2',
            turn: 3,
            gameStatus: { state: 'playing', winner: null, tie: false }
          },
          zones: {
            board: {
              cells: [
                [{ entity: 'mark_p1' }, null, { entity: 'mark_p1' }],
                [null, null, null],
                [{ entity: 'mark_p2' }, null, null]
              ]
            }
          }
        }
      };

      // Reconstruct state from welcome message
      const reconstructedState = {
        you: midGameWelcome.you,
        meta: midGameWelcome.meta,
        state: midGameWelcome.state,
        started: midGameWelcome.started
      };

      // Verify state is correctly reconstructed
      expect(reconstructedState.you).toBe('p2');
      expect(reconstructedState.state.meta.currentPlayer).toBe('p2');
      expect(reconstructedState.state.meta.turn).toBe(3);

      // Verify player can continue playing
      const isYourTurn = reconstructedState.you === reconstructedState.state.meta.currentPlayer;
      const playerActions = reconstructedState.meta.actionMap[reconstructedState.you];

      expect(isYourTurn).toBe(true);
      expect(Object.keys(playerActions)).toHaveLength(2);
    });

    it('should handle out-of-order message delivery', () => {
      let gameState = {
        meta: { tick: 1 },
        state: { meta: { currentPlayer: 'p1' } }
      };

      // Messages arriving out of order
      const messages = [
        { type: 'diff', tick: 3, patch: [{ op: 'replace', path: '/state/meta/currentPlayer', value: 'p1' }] },
        { type: 'diff', tick: 2, patch: [{ op: 'replace', path: '/state/meta/turn', value: 1 }] },
        { type: 'diff', tick: 4, patch: [{ op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' }] }
      ];

      // Sort by tick to handle out-of-order delivery
      const sortedMessages = messages.sort((a, b) => a.tick - b.tick);

      // Apply in correct order
      sortedMessages.forEach((msg) => {
        for (const patch of msg.patch) {
          try {
            const result = applyPatch(gameState, [patch]);
            gameState = result.newDocument;
          } catch (error) {
            console.error('Patch failed:', patch);
          }
        }
      });

      expect(gameState.state.meta.currentPlayer).toBe('p2');
      expect(gameState.state.meta.turn).toBe(1);
    });
  });

  describe('Error Recovery', () => {
    it('should recover from partial patch failures', () => {
      let gameState = {
        meta: { actionMap: { p1: {} } },
        state: {
          zones: {
            board: { cells: [[null, null], [null, null]] }
          }
        }
      };

      // Mix of valid and invalid patches
      const patches = [
        { op: 'replace', path: '/state/zones/board/cells/0/0', value: { entity: 'mark_p1' } }, // Valid
        { op: 'replace', path: '/nonexistent/path', value: 'test' }, // Invalid
        { op: 'replace', path: '/meta/actionMap/p1', value: {} }, // Valid
        { op: 'add', path: '/state/missing/parent/child', value: 'test' } // Invalid
      ];

      let successfulPatches = 0;
      let failedPatches = 0;

      for (const patch of patches) {
        try {
          const result = applyPatch(gameState, [patch]);
          gameState = result.newDocument;
          successfulPatches++;
        } catch (error) {
          failedPatches++;
          // Continue processing other patches
        }
      }

      expect(successfulPatches).toBe(2);
      expect(failedPatches).toBe(2);
      expect(gameState.state.zones.board.cells[0][0]).toEqual({ entity: 'mark_p1' });
    });

    it('should handle corrupted game state gracefully', () => {
      const corruptedStates = [
        // Missing critical paths
        { meta: null, state: { zones: {} } },
        // Wrong data types
        { meta: 'invalid', state: [] },
        // Circular references (would be handled by JSON.stringify)
        { meta: {}, state: { meta: { gameStatus: 'invalid' } } }
      ];

      corruptedStates.forEach((corruptedState, index) => {
        expect(() => {
          // Defensive programming - all accesses should be safe
          const currentPlayer = corruptedState.state?.meta?.currentPlayer;
          const actionMap = corruptedState.meta?.actionMap?.p1 || {};
          const gameStatus = corruptedState.state?.meta?.gameStatus;

          // These should not crash
          const isPlaying = gameStatus?.state === 'playing';
          const hasActions = Object.keys(actionMap).length > 0;
        }, `Corrupted state ${index} should not crash`).not.toThrow();
      });
    });
  });

  describe('Performance Under Load', () => {
    it('should handle rapid state updates efficiently', () => {
      let gameState = {
        meta: { actionMap: { p1: {}, p2: {} } },
        state: { meta: { currentPlayer: 'p1', turn: 0 } }
      };

      const startTime = performance.now();

      // Simulate rapid fire updates
      for (let i = 0; i < 100; i++) {
        const patches = [
          { op: 'replace', path: '/state/meta/turn', value: i },
          { op: 'replace', path: '/state/meta/currentPlayer', value: i % 2 === 0 ? 'p1' : 'p2' },
          { op: 'replace', path: `/meta/actionMap/p${(i % 2) + 1}`, value: {} }
        ];

        for (const patch of patches) {
          try {
            const result = applyPatch(gameState, [patch]);
            gameState = result.newDocument;
          } catch (error) {
            // Skip failed patches
          }
        }
      }

      const endTime = performance.now();
      const duration = endTime - startTime;

      // Should complete quickly (under 50ms for 300 patches)
      expect(duration).toBeLessThan(50);
      expect(gameState.state.meta.turn).toBe(99);
    });

    it('should handle large board states efficiently', () => {
      // Simulate a very large board (much bigger than tic-tac-toe)
      const largeBoardSize = 20;
      const largeBoard = Array(largeBoardSize).fill(null).map(() => 
        Array(largeBoardSize).fill(null)
      );

      // Fill half the board
      for (let row = 0; row < largeBoardSize; row++) {
        for (let col = 0; col < largeBoardSize / 2; col++) {
          largeBoard[row][col] = { entity: `mark_p${(row + col) % 2 + 1}` };
        }
      }

      let gameState = {
        state: {
          zones: {
            board: { cells: largeBoard }
          }
        }
      };

      const startTime = performance.now();

      // Access board state multiple times
      for (let i = 0; i < 100; i++) {
        const board = gameState.state.zones.board.cells;
        const cellCount = board.reduce((acc: number, row: any[]) => 
          acc + row.filter(cell => cell !== null).length, 0
        );
      }

      const endTime = performance.now();
      const duration = endTime - startTime;

      // Should handle large boards efficiently
      expect(duration).toBeLessThan(20);
    });
  });

  describe('Multi-Player Scenarios', () => {
    it('should handle spectator mode correctly', () => {
      const spectatorState = {
        you: 'spectator',
        state: {
          meta: {
            currentPlayer: 'p1',
            gameStatus: { state: 'playing' }
          }
        },
        meta: {
          players: ['ben', 'karen'],
          actionMap: {
            p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } },
            p2: {}
          }
        }
      };

      // Spectator should not be able to perform actions
      const isYourTurn = spectatorState.you === spectatorState.state.meta.currentPlayer;
      const canSpectate = spectatorState.you === 'spectator';
      const playerActions = spectatorState.meta.actionMap[spectatorState.you] || {};

      expect(isYourTurn).toBe(false);
      expect(canSpectate).toBe(true);
      expect(Object.keys(playerActions)).toHaveLength(0);
    });

    it('should handle player disconnection gracefully', () => {
      let gameState = {
        meta: {
          players: ['ben', 'karen'],
          actionMap: { p1: {}, p2: {} }
        },
        state: {
          meta: { currentPlayer: 'p2' }
        }
      };

      // Simulate player disconnection
      const disconnectionUpdate = {
        type: 'playerUpdate',
        players: ['ben'] // karen disconnected
      };

      // Update player list
      gameState = {
        ...gameState,
        meta: {
          ...gameState.meta,
          players: disconnectionUpdate.players
        }
      };

      expect(gameState.meta.players).toEqual(['ben']);
      expect(gameState.meta.players).toHaveLength(1);
    });
  });
});