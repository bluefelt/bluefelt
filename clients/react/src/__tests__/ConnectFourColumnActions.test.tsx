import { describe, it, expect, vi } from 'vitest';
import { useGameActions } from '../hooks/useGameActions';
import { renderHook } from '@testing-library/react';

describe('Connect Four Column Actions', () => {
  it('should handle column clicks correctly', () => {
    const mockSendMessage = vi.fn().mockReturnValue(true);
    const lobbyState = {
      you: 'p1',
      ui: {
        actionMap: {
          'p1': {
            '/zones/board/columns/3': {
              action: 'dropDisc',
              direction: 'Drop disc in column 4'
            }
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Simulate clicking column 3 (row -1 indicates column action)
    result.current.handleCellClick(-1, 3);

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'dropDisc',
        args: {
          zone: '/zones/board',
          column: 3,
          entity: 'disc_p1'
        }
      })
    );
  });

  it('should not send message when not your turn', () => {
    const mockSendMessage = vi.fn();
    const lobbyState = {
      you: 'p1',
      ui: {
        actionMap: {
          'p1': {
            '/zones/board/columns/3': {
              action: 'dropDisc',
              direction: 'Drop disc in column 4'
            }
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: false,
        lobbyState,
        sendMessage: mockSendMessage
      })
    );

    result.current.handleCellClick(-1, 3);

    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('should not send message when column action not available', () => {
    const mockSendMessage = vi.fn();
    const lobbyState = {
      you: 'p1',
      ui: {
        actionMap: {
          'p1': {
            // No column actions available
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState,
        sendMessage: mockSendMessage
      })
    );

    result.current.handleCellClick(-1, 3);

    expect(mockSendMessage).not.toHaveBeenCalled();
  });

  it('should handle regular cell clicks for non-gravity games', () => {
    const mockSendMessage = vi.fn().mockReturnValue(true);
    const lobbyState = {
      you: 'p1',
      ui: {
        actionMap: {
          'p1': {
            '/zones/board/cells/2/3': {
              action: 'place',
              direction: 'Place mark'
            }
          }
        }
      }
    };

    const { result } = renderHook(() => 
      useGameActions({
        isYourTurn: true,
        lobbyState,
        sendMessage: mockSendMessage
      })
    );

    // Regular cell click
    result.current.handleCellClick(2, 3);

    expect(mockSendMessage).toHaveBeenCalledWith(
      JSON.stringify({
        action: 'place',
        args: {
          location: '/zones/board/cells/2/3',
          entity: 'mark_p1'
        }
      })
    );
  });
});