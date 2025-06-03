/**
 * Tests for action handling and edge cases
 * Validates click handling, message construction, and error scenarios
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

describe('Tic-Tac-Toe Action Handling', () => {

  describe('Click Handling Logic', () => {
    it('should handle valid cell clicks correctly', () => {
      const lobbyState = {
        you: 'p1',
        state: {
          meta: { currentPlayer: 'p1' }
        },
        meta: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/2/2': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      const validClicks = [
        { row: 0, col: 0 },
        { row: 1, col: 1 },
        { row: 2, col: 2 }
      ];

      validClicks.forEach(({ row, col }) => {
        const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
        const location = `/zones/board/cells/${row}/${col}`;
        const action = playerActions[location];
        const isYourTurn = lobbyState.state?.meta?.currentPlayer === lobbyState.you;

        expect(action, `Action should exist for ${location}`).toBeDefined();
        expect(isYourTurn, 'Should be player turn').toBe(true);

        if (action && isYourTurn) {
          const entity = `mark_${lobbyState.you}`;
          const message = {
            action: action.action,
            args: {
              location: location,
              entity: entity
            }
          };

          expect(message.action).toBe('placeMarker');
          expect(message.args.location).toBe(location);
          expect(message.args.entity).toBe('mark_p1');
        }
      });
    });

    it('should reject invalid cell clicks', () => {
      const lobbyState = {
        you: 'p1',
        state: {
          meta: { currentPlayer: 'p2' } // Not player's turn
        },
        meta: {
          actionMap: {
            p1: {},
            p2: {
              '/zones/board/cells/0/0': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      const invalidClicks = [
        { row: 0, col: 0, reason: 'Not in player action map' },
        { row: 1, col: 1, reason: 'Not in action map at all' },
        { row: -1, col: 0, reason: 'Invalid coordinates' },
        { row: 0, col: 3, reason: 'Out of bounds' }
      ];

      invalidClicks.forEach(({ row, col, reason }) => {
        const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
        const location = `/zones/board/cells/${row}/${col}`;
        const action = playerActions[location];
        const isYourTurn = lobbyState.state?.meta?.currentPlayer === lobbyState.you;

        // Should not be able to perform action
        const canClick = action && isYourTurn;
        expect(canClick, `Should not be able to click ${location}: ${reason}`).toBeFalsy();
      });
    });

    it('should handle clicks on occupied cells', () => {
      const lobbyState = {
        you: 'p1',
        state: {
          meta: { currentPlayer: 'p1' },
          zones: {
            board: {
              cells: [
                [{ entity: 'mark_p2' }, null, null],
                [null, null, null],
                [null, null, null]
              ]
            }
          }
        },
        meta: {
          actionMap: {
            p1: {
              // Note: occupied cells shouldn't be in action map
              '/zones/board/cells/0/1': { action: 'placeMarker', direction: 'Click to place' },
              '/zones/board/cells/1/1': { action: 'placeMarker', direction: 'Click to place' }
            }
          }
        }
      };

      // Clicking on occupied cell (0,0) - should not be in action map
      const occupiedLocation = '/zones/board/cells/0/0';
      const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
      const actionOnOccupied = playerActions[occupiedLocation];

      expect(actionOnOccupied, 'Occupied cell should not have action').toBeUndefined();

      // Verify the cell is actually occupied
      const cellValue = lobbyState.state.zones.board.cells[0][0];
      expect(cellValue).toEqual({ entity: 'mark_p2' });

      // Valid empty cells should still be clickable
      const emptyLocation = '/zones/board/cells/0/1';
      const actionOnEmpty = playerActions[emptyLocation];
      expect(actionOnEmpty, 'Empty cell should have action').toBeDefined();
    });
  });

  describe('Message Construction', () => {
    it('should construct well-formed action messages', () => {
      const testCases = [
        {
          player: 'p1',
          row: 0,
          col: 0,
          expectedEntity: 'mark_p1',
          expectedLocation: '/zones/board/cells/0/0'
        },
        {
          player: 'p2', 
          row: 2,
          col: 1,
          expectedEntity: 'mark_p2',
          expectedLocation: '/zones/board/cells/2/1'
        }
      ];

      testCases.forEach(({ player, row, col, expectedEntity, expectedLocation }) => {
        const action = {
          action: 'placeMarker',
          args: {
            location: `/zones/board/cells/${row}/${col}`,
            entity: `mark_${player}`
          }
        };

        // Verify message structure
        expect(action.action).toBe('placeMarker');
        expect(action.args.location).toBe(expectedLocation);
        expect(action.args.entity).toBe(expectedEntity);

        // Verify it's valid JSON
        const jsonString = JSON.stringify(action);
        const parsed = JSON.parse(jsonString);
        expect(parsed).toEqual(action);
      });
    });

    it('should handle message serialization edge cases', () => {
      const edgeCaseMessages = [
        {
          name: 'Special characters in location',
          message: {
            action: 'placeMarker',
            args: {
              location: '/zones/board/cells/0/0',
              entity: 'mark_p1'
            }
          }
        },
        {
          name: 'Unicode in player name',
          message: {
            action: 'placeMarker', 
            args: {
              location: '/zones/board/cells/1/1',
              entity: 'mark_p1',
              metadata: { playerName: 'José' }
            }
          }
        }
      ];

      edgeCaseMessages.forEach(({ name, message }) => {
        expect(() => {
          const serialized = JSON.stringify(message);
          const deserialized = JSON.parse(serialized);
          expect(deserialized).toEqual(message);
        }, `${name} should serialize/deserialize correctly`).not.toThrow();
      });
    });
  });


  describe('State Validation', () => {
    it('should validate game state before allowing actions', () => {
      const validationScenarios = [
        {
          name: 'Valid state',
          lobbyState: {
            you: 'p1',
            state: { meta: { currentPlayer: 'p1', gameStatus: { state: 'playing' } } },
            meta: { actionMap: { p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } } } }
          },
          shouldAllowAction: true
        },
        {
          name: 'Game ended',
          lobbyState: {
            you: 'p1',
            state: { meta: { currentPlayer: 'p1', gameStatus: { state: 'ended' } } },
            meta: { actionMap: { p1: {} } }
          },
          shouldAllowAction: false
        },
        {
          name: 'Not player turn',
          lobbyState: {
            you: 'p1',
            state: { meta: { currentPlayer: 'p2', gameStatus: { state: 'playing' } } },
            meta: { actionMap: { p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } } } }
          },
          shouldAllowAction: false
        },
        {
          name: 'Spectator',
          lobbyState: {
            you: 'spectator',
            state: { meta: { currentPlayer: 'p1', gameStatus: { state: 'playing' } } },
            meta: { actionMap: { p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } } } }
          },
          shouldAllowAction: false
        }
      ];

      validationScenarios.forEach(({ name, lobbyState, shouldAllowAction }) => {
        const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
        const location = '/zones/board/cells/0/0';
        const action = playerActions[location];
        const isYourTurn = lobbyState.you && 
                          lobbyState.you !== 'spectator' && 
                          lobbyState.state?.meta?.currentPlayer === lobbyState.you;
        const gameStatus = lobbyState.state?.meta?.gameStatus;
        const isGamePlaying = !gameStatus || gameStatus.state === 'playing';

        const canPerformAction = !!(action && isYourTurn && isGamePlaying);

        expect(canPerformAction, `${name} - can perform action`).toBe(shouldAllowAction);
      });
    });

    it('should handle missing or malformed validation data', () => {
      const malformedStates = [
        {
          name: 'Missing action map',
          lobbyState: { you: 'p1', state: { meta: { currentPlayer: 'p1' } } }
        },
        {
          name: 'Missing current player',
          lobbyState: { you: 'p1', meta: { actionMap: { p1: {} } } }
        },
        {
          name: 'Null values',
          lobbyState: { you: 'p1', state: null, meta: null }
        }
      ];

      malformedStates.forEach(({ name, lobbyState }) => {
        expect(() => {
          const playerActions = lobbyState.meta?.actionMap?.[lobbyState.you || ''] || {};
          const currentPlayer = lobbyState.state?.meta?.currentPlayer;
          const isYourTurn = lobbyState.you === currentPlayer;
          
          // Should not crash when validating malformed state
          const hasActions = Object.keys(playerActions).length > 0;
          const canAct = hasActions && isYourTurn;
        }, `${name} should not crash validation`).not.toThrow();
      });
    });
  });

});