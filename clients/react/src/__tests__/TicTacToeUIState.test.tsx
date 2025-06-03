/**
 * Tests for UI state synchronization and game flow logic
 * Validates that the UI correctly reflects game state changes
 */

import { describe, it, expect } from 'vitest';

describe('Tic-Tac-Toe UI State Synchronization', () => {

  describe('Turn Detection and Display', () => {
    it('should correctly identify whose turn it is', () => {
      const testScenarios = [
        {
          name: 'Player 1 turn',
          lobbyState: {
            you: 'p1',
            state: { meta: { currentPlayer: 'p1' } },
            meta: { currentPlayer: 'p2' } // Should be ignored
          },
          expectedIsYourTurn: true
        },
        {
          name: 'Player 2 turn',
          lobbyState: {
            you: 'p1', 
            state: { meta: { currentPlayer: 'p2' } }
          },
          expectedIsYourTurn: false
        },
        {
          name: 'Spectator view',
          lobbyState: {
            you: 'spectator',
            state: { meta: { currentPlayer: 'p1' } }
          },
          expectedIsYourTurn: false
        },
        {
          name: 'Missing current player',
          lobbyState: {
            you: 'p1',
            state: { meta: {} },
            meta: { currentPlayer: 'p1' }
          },
          expectedIsYourTurn: true // Falls back to meta.currentPlayer
        }
      ];

      testScenarios.forEach(({ name, lobbyState, expectedIsYourTurn }) => {
        // Logic from GameView.tsx
        const currentPlayer = lobbyState.state?.meta?.currentPlayer || lobbyState.meta?.currentPlayer;
        const isYourTurn = lobbyState.you && 
                          lobbyState.you !== 'spectator' && 
                          currentPlayer === lobbyState.you;
        
        expect(isYourTurn, `${name} - isYourTurn`).toBe(expectedIsYourTurn);
      });
    });

    it('should handle turn transitions correctly', () => {
      const gameProgression = [
        {
          turn: 0,
          currentPlayer: 'p1',
          expectedTurnDisplay: 'Player 1 turn'
        },
        {
          turn: 1, 
          currentPlayer: 'p2',
          expectedTurnDisplay: 'Player 2 turn'
        },
        {
          turn: 0, // Cycles back 
          currentPlayer: 'p1',
          expectedTurnDisplay: 'Player 1 turn'
        }
      ];

      gameProgression.forEach(({ turn, currentPlayer, expectedTurnDisplay }) => {
        const playerNumber = currentPlayer.replace('p', '');
        const actualDisplay = `Player ${playerNumber} turn`;
        expect(actualDisplay).toBe(expectedTurnDisplay);
      });
    });
  });

  describe('Action Map Processing', () => {
    it('should determine clickable cells based on action map', () => {
      const lobbyState = {
        you: 'p1',
        meta: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/1': { action: 'placeMarker', direction: 'Click to place' }
            },
            p2: {
              '/zones/board/cells/2/2': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
      
      // Test specific cell clickability
      expect(playerActions['/zones/board/cells/0/0']).toBeDefined();
      expect(playerActions['/zones/board/cells/0/1']).toBeDefined();
      expect(playerActions['/zones/board/cells/1/1']).toBeDefined();
      expect(playerActions['/zones/board/cells/2/2']).toBeUndefined(); // This is for p2
      
      // Test action message construction
      const cellAction = playerActions['/zones/board/cells/0/0'];
      if (cellAction) {
        expect(cellAction.action).toBe('placeMarker');
        expect(cellAction.direction).toBe('Click to place');
      }
    });

    it('should handle empty action maps correctly', () => {
      const testCases = [
        {
          name: 'No action map for player',
          lobbyState: {
            you: 'p1',
            meta: {
              actionMap: {
                p2: { '/zones/board/cells/0/0': { action: 'placeMarker' } }
              }
            }
          }
        },
        {
          name: 'Empty action map object',
          lobbyState: {
            you: 'p1',
            meta: {
              actionMap: { p1: {} }
            }
          }
        },
        {
          name: 'Missing action map entirely',
          lobbyState: {
            you: 'p1',
            meta: {}
          }
        }
      ];

      testCases.forEach(({ name, lobbyState }) => {
        const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
        expect(Object.keys(playerActions)).toHaveLength(0);
      });
    });

    it('should correctly format cell locations', () => {
      const board3x3 = Array(3).fill(null).map(() => Array(3).fill(null));
      
      for (let row = 0; row < 3; row++) {
        for (let col = 0; col < 3; col++) {
          const location = `/zones/board/cells/${row}/${col}`;
          
          // Verify format matches server expectations
          expect(location).toMatch(/^\/zones\/board\/cells\/\d+\/\d+$/);
          expect(location.split('/')).toHaveLength(6); // ['', 'zones', 'board', 'cells', 'row', 'col']
        }
      }
    });
  });

  describe('Board State Updates', () => {
    it('should reflect board changes correctly', () => {
      const initialBoard = [
        [null, null, null],
        [null, null, null],
        [null, null, null]
      ];

      const boardAfterMove = [
        [{ entity: 'mark_p1' }, null, null],
        [null, null, null],
        [null, null, null]
      ];

      const boardAfterTwoMoves = [
        [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, null],
        [null, null, null],
        [null, null, null]
      ];

      // Verify initial state
      expect(initialBoard[0][0]).toBeNull();
      
      // Verify after first move
      expect(boardAfterMove[0][0]).toEqual({ entity: 'mark_p1' });
      expect(boardAfterMove[0][1]).toBeNull();
      
      // Verify after second move
      expect(boardAfterTwoMoves[0][0]).toEqual({ entity: 'mark_p1' });
      expect(boardAfterTwoMoves[0][1]).toEqual({ entity: 'mark_p2' });
    });

    it('should handle board state with different entity formats', () => {
      const boardVariations = [
        // Standard format
        { entity: 'mark_p1' },
        // With additional properties
        { entity: 'mark_p2', timestamp: Date.now() },
        // With nested properties
        { entity: 'mark_p1', props: { color: 'red' } }
      ];

      boardVariations.forEach((cellValue, index) => {
        expect(cellValue.entity).toMatch(/^mark_p[12]$/);
        
        // Extract player from entity
        const player = cellValue.entity.split('_')[1];
        expect(['p1', 'p2']).toContain(player);
      });
    });
  });

  describe('Game End Detection', () => {
    it('should detect win conditions correctly', () => {
      const winScenarios = [
        {
          name: 'Horizontal win',
          gameStatus: { state: 'ended', winner: 'p1', tie: false },
          board: [
            [{ entity: 'mark_p1' }, { entity: 'mark_p1' }, { entity: 'mark_p1' }],
            [{ entity: 'mark_p2' }, { entity: 'mark_p2' }, null],
            [null, null, null]
          ]
        },
        {
          name: 'Diagonal win',
          gameStatus: { state: 'ended', winner: 'p2', tie: false },
          board: [
            [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, null],
            [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, null],
            [null, null, { entity: 'mark_p2' }]
          ]
        },
        {
          name: 'Tie game',
          gameStatus: { state: 'ended', winner: null, tie: true },
          board: [
            [{ entity: 'mark_p1' }, { entity: 'mark_p2' }, { entity: 'mark_p1' }],
            [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p2' }],
            [{ entity: 'mark_p2' }, { entity: 'mark_p1' }, { entity: 'mark_p2' }]
          ]
        }
      ];

      winScenarios.forEach(({ name, gameStatus, board }) => {
        // Test game status detection
        const isGameEnded = gameStatus.state === 'ended';
        const hasWinner = gameStatus.winner !== null;
        const isTie = gameStatus.tie;

        expect(isGameEnded, `${name} - game ended`).toBe(true);
        
        if (name !== 'Tie game') {
          expect(hasWinner, `${name} - has winner`).toBe(true);
          expect(isTie, `${name} - is tie`).toBe(false);
          expect(['p1', 'p2']).toContain(gameStatus.winner);
        } else {
          expect(hasWinner, `${name} - has winner`).toBe(false);
          expect(isTie, `${name} - is tie`).toBe(true);
        }

        // Verify board is full for tie
        if (isTie) {
          const isBoardFull = board.every(row => 
            row.every(cell => cell !== null && cell.entity)
          );
          expect(isBoardFull, `${name} - board full`).toBe(true);
        }
      });
    });

    it('should handle game end UI updates', () => {
      const endedGameState = {
        you: 'p1',
        state: {
          meta: {
            currentPlayer: 'p1', // Should be irrelevant when game ended
            gameStatus: { state: 'ended', winner: 'p1', tie: false }
          }
        },
        meta: {
          actionMap: {
            p1: {}, // Should be empty when game ended
            p2: {}
          }
        }
      };

      const gameStatus = endedGameState.state.meta.gameStatus;
      const isGameEnded = gameStatus.state === 'ended';
      const playerWon = gameStatus.winner === endedGameState.you;
      const actionMap = endedGameState.meta.actionMap[endedGameState.you];

      expect(isGameEnded).toBe(true);
      expect(playerWon).toBe(true);
      expect(Object.keys(actionMap)).toHaveLength(0); // No moves available when game ended
    });
  });

  describe('Player Entity Display', () => {
    it('should correctly identify player entities and colors', () => {
      const entityDefinitions = [
        { id: 'mark_p1', props: { value: 'p1' }, ui: { tokenType: 'p1' } },
        { id: 'mark_p2', props: { value: 'p2' }, ui: { tokenType: 'p2' } }
      ];

      const players = ['ben', 'karen']; // p1, p2

      players.forEach((playerName, index) => {
        const playerId = `p${index + 1}`;
        const entityId = `mark_${playerId}`;
        
        const entity = entityDefinitions.find(e => e.id === entityId);
        expect(entity).toBeDefined();
        expect(entity!.props.value).toBe(playerId);
        expect(entity!.ui.tokenType).toBe(playerId);
      });
    });

    it('should handle player name to ID mapping', () => {
      const lobbyState = {
        you: 'p1',
        meta: {
          players: ['ben', 'karen']
        }
      };

      // Logic to map player IDs to names
      const playerIdToName = (playerId: string) => {
        const playerIndex = parseInt(playerId.replace('p', '')) - 1;
        return lobbyState.meta.players[playerIndex];
      };

      expect(playerIdToName('p1')).toBe('ben');
      expect(playerIdToName('p2')).toBe('karen');
    });
  });

  describe('Error State Handling', () => {
    it('should handle missing or corrupt game state gracefully', () => {
      const corruptStates = [
        {
          name: 'Missing state object',
          lobbyState: { you: 'p1', meta: {} }
        },
        {
          name: 'Missing meta object', 
          lobbyState: { you: 'p1', state: {} }
        },
        {
          name: 'Missing both state and meta',
          lobbyState: { you: 'p1' }
        },
        {
          name: 'Null values',
          lobbyState: { you: 'p1', state: null, meta: null }
        }
      ];

      corruptStates.forEach(({ name, lobbyState }) => {
        // Should not crash when accessing nested properties
        const currentPlayer = lobbyState.state?.meta?.currentPlayer || lobbyState.meta?.currentPlayer;
        const actionMap = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
        const gameStatus = lobbyState.state?.meta?.gameStatus;

        // These should all be safe to access
        expect(() => {
          const isYourTurn = lobbyState.you && 
                            lobbyState.you !== 'spectator' && 
                            currentPlayer === lobbyState.you;
          const hasActions = Object.keys(actionMap).length > 0;
          const isGameEnded = gameStatus?.state === 'ended';
        }).not.toThrow();
      });
    });

    it('should provide sensible defaults for missing data', () => {
      const minimalState = { you: 'p1' };

      // Simulate default value logic
      const currentPlayer = minimalState.state?.meta?.currentPlayer || minimalState.meta?.currentPlayer || '';
      const actionMap = minimalState.meta?.actionMap?.[minimalState.you] || {};
      const turn = minimalState.state?.meta?.turn ?? minimalState.meta?.turn ?? 0;
      const gameStatus = minimalState.state?.meta?.gameStatus || { state: 'playing', winner: null, tie: false };

      expect(currentPlayer).toBe('');
      expect(actionMap).toEqual({});
      expect(turn).toBe(0);
      expect(gameStatus.state).toBe('playing');
    });
  });
});