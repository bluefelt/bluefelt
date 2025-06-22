import { describe, it, expect, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useGameActions } from '../hooks/useGameActions';

describe('Client Request Generation Tests', () => {
  const mockSendMessage = vi.fn();

  beforeEach(() => {
    mockSendMessage.mockClear();
  });

  describe('Tic-Tac-Toe Actions', () => {
    it('should generate correct placeMarker request', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          entities: [
            { id: 'mark_p1', props: { value: 'p1' } },
            { id: 'mark_p2', props: { value: 'p2' } },
          ],
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { 
                action: 'placeMarker',
                args: {
                  location: '/zones/board/cells/0/0',
                  entity: 'mark_p1',
                  player: 'p1'
                }
              },
            },
          },
        },
        game: {
          currentPlayer: 'p1',
        },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'placeMarker',
          args: {
            location: '/zones/board/cells/0/0',
            entity: 'mark_p1',
            player: 'p1'
          },
        })
      );
    });

    it('should not send action when not your turn', () => {
      const lobbyState = {
        you: 'p1',
        ui: { actionMap: {} },
        game: { currentPlayer: 'p2' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: false,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).not.toHaveBeenCalled();
    });
  });

  describe('Go Fish Actions', () => {
    it('should generate correct selectRank request', () => {
      const lobbyState = {
        you: 'p1',
        game: {
          phases: { game: { current: 'selectingRank' } },
          currentPlayer: 'p1',
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/choice_p1/ranks/A': {
                action: 'selectRank',
                direction: 'Choose a rank to ask for',
                args: {
                  rank: 'A',
                  location: '/zones/choice_p1/ranks/A',
                  player: 'p1'
                }
              }
            }
          }
        }
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleChoiceSelect('choice_p1/ranks', 'A');
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'selectRank',
          args: {
            rank: 'A',
            location: '/zones/choice_p1/ranks/A',
            player: 'p1',
          },
        })
      );
    });

    it('should generate correct selectPlayer request', () => {
      const lobbyState = {
        you: 'p1',
        game: {
          phases: { game: { current: 'selectingPlayer' } },
          currentPlayer: 'p1',
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/choice_p1/players/p2': {
                action: 'selectPlayer',
                direction: 'Choose a player to ask',
                args: {
                  targetPlayer: 'p2',
                  location: '/zones/choice_p1/players/p2',
                  player: 'p1'
                }
              }
            }
          }
        }
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleChoiceSelect('choice_p1/players', 'p2');
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'selectPlayer',
          args: {
            targetPlayer: 'p2',
            location: '/zones/choice_p1/players/p2',
            player: 'p1',
          },
        })
      );
    });
  });

  describe('Three Men\'s Morris Actions', () => {
    it('should generate correct selectPiece request', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { 
                action: 'selectPiece',
                args: {
                  target: '/zones/board/cells/0/0',
                  location: '/zones/board/cells/0/0',
                  player: 'p1'
                }
              },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'selectPiece',
          args: {
            target: '/zones/board/cells/0/0',
            location: '/zones/board/cells/0/0',
            player: 'p1',
          },
        })
      );
    });

    it('should generate correct moveSelectedPiece request', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/1/1': { 
                action: 'moveSelectedPiece',
                args: {
                  target: '/zones/board/cells/1/1',
                  location: '/zones/board/cells/1/1',
                  player: 'p1'
                }
              },
            },
          },
        },
        game: { 
          currentPlayer: 'p1',
          selection: { piece: '/zones/board/cells/0/0' },
        },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(1, 1);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'moveSelectedPiece',
          args: {
            target: '/zones/board/cells/1/1',
            location: '/zones/board/cells/1/1',
            player: 'p1',
          },
        })
      );
    });

    it('should generate correct clearSelection request', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '_global': { 
                action: 'clearSelection',
                args: {
                  player: 'p1'
                }
              },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleZoneAction('_global', 'clearSelection');
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'clearSelection',
          args: {
            player: 'p1',
          },
        })
      );
    });
  });

  describe('Connect Four Actions', () => {
    it('should generate correct column-based action', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          entities: [
            { id: 'disc_p1', props: { value: 'p1' } },
            { id: 'disc_p2', props: { value: 'p2' } },
          ],
          actionMap: {
            p1: {
              '/zones/board/columns/3': { 
                action: 'dropDisc', 
                args: {
                  targetColumn: 3,
                  location: '/zones/board/columns/3',
                  player: 'p1'
                }
              },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      // Column actions use row = -1
      act(() => {
        result.current.handleCellClick(-1, 3);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'dropDisc',
          args: {
            targetColumn: 3,
            location: '/zones/board/columns/3',
            player: 'p1',
          },
        })
      );
    });
  });

  describe('Generic Actions', () => {
    it('should handle zone-level actions', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '/zones/deck': { 
                action: 'drawCard',
                args: {
                  source: 'deck',
                  location: '/zones/deck',
                  player: 'p1'
                }
              },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCardAction('deck', -1);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'drawCard',
          args: { 
            source: 'deck',
            location: '/zones/deck',
            player: 'p1'
          },
        })
      );
    });

    it('should handle card-specific actions', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '/zones/hand_p1/items/2': { 
                action: 'playCard',
                args: { 
                  entity: 'card_hearts_a',
                  location: '/zones/hand_p1/items/2',
                  player: 'p1'
                },
              },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCardAction('hand_p1', 2);
      });

      expect(mockSendMessage).toHaveBeenCalledWith(
        JSON.stringify({
          action: 'playCard',
          args: { 
            entity: 'card_hearts_a',
            location: '/zones/hand_p1/items/2',
            player: 'p1'
          },
        })
      );
    });
  });

  describe('Error Handling', () => {
    it('should handle missing player ID gracefully', () => {
      const lobbyState = {
        you: null, // No player ID
        ui: { actionMap: {} },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).not.toHaveBeenCalled();
    });

    it('should handle missing action map gracefully', () => {
      const lobbyState = {
        you: 'p1',
        ui: {}, // No action map
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).not.toHaveBeenCalled();
    });

    it('should validate action exists before sending', () => {
      const lobbyState = {
        you: 'p1',
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMarker' },
            },
          },
        },
        game: { currentPlayer: 'p1' },
      };

      const { result } = renderHook(() => 
        useGameActions({
          isYourTurn: true,
          lobbyState,
          sendMessage: mockSendMessage,
        })
      );

      // Try to click cell that has no action
      act(() => {
        result.current.handleCellClick(1, 1);
      });

      expect(mockSendMessage).not.toHaveBeenCalled();

      // Click cell with action
      act(() => {
        result.current.handleCellClick(0, 0);
      });

      expect(mockSendMessage).toHaveBeenCalledTimes(1);
    });
  });
});