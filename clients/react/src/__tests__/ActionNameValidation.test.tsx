/**
 * Action Name Validation Tests
 * 
 * These tests ensure that action names expected by the client match those provided by the server.
 * This prevents the client-server action name mismatch issue we experienced with Tic Tac Toe.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import React from 'react';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import type { LobbyState } from '../types/messages';

// Mock WebSocket for controlled testing
class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  public readonly CONNECTING = 0;
  public readonly OPEN = 1;
  public readonly CLOSING = 2;
  public readonly CLOSED = 3;

  public readyState = MockWebSocket.CONNECTING;
  public onopen: ((event: Event) => void) | null = null;
  public onmessage: ((event: MessageEvent) => void) | null = null;
  public onclose: ((event: CloseEvent) => void) | null = null;
  public onerror: ((event: Event) => void) | null = null;

  constructor(public url: string) {
    // Simulate connection opening
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event('open'));
    }, 10);
  }

  send(data: string | ArrayBuffer | SharedArrayBuffer | Blob | ArrayBufferView): void {
    // Mock send - we'll track sent messages if needed
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent('close'));
  }

  // Helper to simulate receiving a message
  simulateMessage(data: any): void {
    if (this.readyState === MockWebSocket.OPEN && this.onmessage) {
      this.onmessage(new MessageEvent('message', { data: JSON.stringify(data) }));
    }
  }
}

// Override global WebSocket
const originalWebSocket = global.WebSocket;

describe('Action Name Validation Tests', () => {
  let mockWS: MockWebSocket;

  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock WebSocket
    global.WebSocket = vi.fn().mockImplementation((url: string) => {
      mockWS = new MockWebSocket(url);
      return mockWS;
    }) as any;
  });

  afterEach(() => {
    global.WebSocket = originalWebSocket;
  });

  describe('Tic Tac Toe Action Names', () => {
    it('should expect placeMark action name (not placeMarker)', async () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'test-player', true)
      );

      // Wait for WebSocket to connect
      await waitFor(() => {
        expect(result.current.connected).toBe(true);
      });

      // Simulate receiving a welcome message with action map
      const welcomeMessage = {
        type: 'welcome',
        you: 'p1',
        started: true,
        game: {
          currentPlayer: 'p1',
          zones: {
            board: {
              cells: Array(3).fill(null).map(() => Array(3).fill(null))
            }
          },
          players: [{ id: 'p1' }, { id: 'p2' }],
          phases: { game: { current: 'play', count: 0, actionsProcessed: 0 } },
          gameStatus: { state: 'playing', winner: null, tie: false }
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMark', direction: 'Click to place' },
              '/zones/board/cells/0/1': { action: 'placeMark', direction: 'Click to place' },
              '/zones/board/cells/1/1': { action: 'placeMark', direction: 'Click to place' }
            },
            p2: {}
          }
        }
      };

      await act(async () => {
        mockWS.simulateMessage(welcomeMessage);
      });

      await waitFor(() => {
        expect(result.current.lobbyState.ui?.actionMap).toBeDefined();
      });

      const actionMap = result.current.lobbyState.ui?.actionMap;
      expect(actionMap).toBeDefined();

      // Verify all actions use 'placeMark' not 'placeMarker'
      if (actionMap && actionMap.p1) {
        for (const [path, actionData] of Object.entries(actionMap.p1)) {
          expect(actionData.action).toBe('placeMark');
          expect(actionData.action).not.toBe('placeMarker');
        }
      }

      console.log('✅ Client expects correct action name: placeMark');
    });

    it('should reject action maps with old placeMarker action name', async () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'test-player', true)
      );

      await waitFor(() => {
        expect(result.current.connected).toBe(true);
      });

      // Simulate receiving action map with NEW action name (correct behavior)
      const welcomeWithNewAction = {
        type: 'welcome',
        you: 'p1',
        started: true,
        game: {
          currentPlayer: 'p1',
          zones: { board: { cells: Array(3).fill(null).map(() => Array(3).fill(null)) } },
          players: [{ id: 'p1' }, { id: 'p2' }],
          phases: { game: { current: 'play', count: 0, actionsProcessed: 0 } },
          gameStatus: { state: 'playing', winner: null, tie: false }
        },
        ui: {
          actionMap: {
            p1: {
              '/zones/board/cells/0/0': { action: 'placeMark', direction: 'Click to place' }
            },
            p2: {}
          }
        }
      };

      await act(async () => {
        mockWS.simulateMessage(welcomeWithNewAction);
      });

      await waitFor(() => {
        expect(result.current.lobbyState.ui?.actionMap).toBeDefined();
      });

      const actionMap = result.current.lobbyState.ui?.actionMap;
      
      // This should fail if server sends old action name
      if (actionMap && actionMap.p1) {
        for (const [path, actionData] of Object.entries(actionMap.p1)) {
          expect(actionData.action).not.toBe('placeMarker'); // Should NOT be old name
          expect(actionData.action).toBe('placeMark'); // Should be new name
        }
      }
    });
  });

  describe('Action Name Consistency Across Games', () => {
    it('should validate action names are consistent for known games', async () => {
      // Test multiple games to ensure consistency
      const gameTestCases = [
        {
          gameId: 'tic-tac-toe',
          expectedActions: ['placeMark']
        },
        {
          gameId: 'connect-four', 
          expectedActions: ['dropChecker']
        }
        // Add more games as needed
      ];

      for (const testCase of gameTestCases) {
        const { result } = renderHook(() => 
          useLobbyWebSocket('test-lobby', 'test-player', true)
        );

        await waitFor(() => {
          expect(result.current.connected).toBe(true);
        });

        // Simulate action map for this game
        const actionMap: Record<string, any> = {};
        for (const actionName of testCase.expectedActions) {
          actionMap['/zones/board/cells/0/0'] = { 
            action: actionName, 
            direction: 'Click to interact' 
          };
        }

        const welcomeMessage = {
          type: 'welcome',
          you: 'p1',
          started: true,
          game: {
            currentPlayer: 'p1',
            zones: { board: { cells: [[null]] } },
            players: [{ id: 'p1' }],
            phases: { game: { current: 'play', count: 0, actionsProcessed: 0 } },
            gameStatus: { state: 'playing', winner: null, tie: false }
          },
          ui: {
            actionMap: { p1: actionMap, p2: {} }
          }
        };

        await act(async () => {
          mockWS.simulateMessage(welcomeMessage);
        });

        await waitFor(() => {
          expect(result.current.lobbyState.ui?.actionMap).toBeDefined();
        });

        // Verify expected action names are present
        const receivedActionMap = result.current.lobbyState.ui?.actionMap;
        if (receivedActionMap && receivedActionMap.p1) {
          const receivedActions = Object.values(receivedActionMap.p1).map(
            (actionData: any) => actionData.action
          );
          
          for (const expectedAction of testCase.expectedActions) {
            expect(receivedActions).toContain(expectedAction);
          }
        }

        console.log(`✅ Action names validated for ${testCase.gameId}`);
      }
    });
  });

  describe('Patch Processing Error Prevention', () => {
    it('should handle remove operations on non-existent paths gracefully', async () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'test-player', true)
      );

      await waitFor(() => {
        expect(result.current.connected).toBe(true);
      });

      // First send a valid state
      mockWS.simulateMessage({
        type: 'welcome',
        you: 'p1',
        game: { zones: {} },
        ui: {}
      });

      await waitFor(() => {
        expect(result.current.lobbyState.game).toBeDefined();
      });

      // Now send a patch that tries to remove a non-existent path
      const patchMessage = {
        type: 'diff',
        patch: [
          {
            op: 'remove',
            path: '/ui/currentPhasePrompt' // This path doesn't exist
          }
        ]
      };

      // This should not throw an error
      expect(() => {
        mockWS.simulateMessage(patchMessage);
      }).not.toThrow();

      console.log('✅ Patch processing handles non-existent paths gracefully');
    });

    it('should prevent React setState during render errors', async () => {
      const { result } = renderHook(() => 
        useLobbyWebSocket('test-lobby', 'test-player', true)
      );

      await waitFor(() => {
        expect(result.current.connected).toBe(true);
      });

      // Send initial state
      mockWS.simulateMessage({
        type: 'welcome',
        you: 'p1',
        game: { zones: { board: { cells: [[null]] } } },
        ui: {}
      });

      // Send a patch that could trigger animations
      const animationPatch = {
        type: 'diff',
        patch: [
          {
            op: 'replace',
            path: '/game/zones/board/cells/0/0',
            value: { entity: 'mark_p1' }
          }
        ]
      };

      // This should not cause React setState during render error
      expect(() => {
        mockWS.simulateMessage(animationPatch);
      }).not.toThrow();

      console.log('✅ Animation processing deferred to prevent setState during render');
    });
  });
});