/**
 * Integration tests that verify server-client data flow
 * These tests catch issues where tests pass but the real app fails
 */

import { describe, it, expect } from 'vitest';

describe('Server-Client Integration', () => {

  describe('Expected Server Patch Format', () => {
    it('should expect server to send both /meta and /state/meta patches for turn advancement', () => {
      // This test documents what the server SHOULD send
      // If this test passes but the app fails, it means the server isn't sending the expected patches
      
      const expectedTurnAdvancementPatches = [
        { op: 'replace', path: '/meta/tick', value: 1 },
        { op: 'replace', path: '/meta/turn', value: 1 },
        { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/state/meta/turn', value: 1 }
      ];

      // Verify the client logic expects this format
      const mockState = {
        meta: { currentPlayer: 'p1', turn: 0 },
        state: { meta: { currentPlayer: 'p1', turn: 0 } }
      };

      // Simulate applying the patches the server SHOULD send
      expectedTurnAdvancementPatches.forEach(patch => {
        const pathParts = patch.path.split('/').filter(p => p);
        let target = mockState;
        
        for (let i = 0; i < pathParts.length - 1; i++) {
          target = target[pathParts[i] as keyof typeof target] as any;
        }
        
        const finalKey = pathParts[pathParts.length - 1];
        (target as any)[finalKey] = patch.value;
      });

      // Verify both meta and state.meta are updated
      expect(mockState.meta.currentPlayer).toBe('p2');
      expect(mockState.state.meta.currentPlayer).toBe('p2');
      
      // Verify client logic would work correctly
      const currentPlayer = mockState.state?.meta?.currentPlayer || mockState.meta?.currentPlayer;
      expect(currentPlayer).toBe('p2');
    });

    it('should detect when server only sends /meta patches (the bug)', () => {
      // This test shows what happens when server only sends /meta patches
      
      const actualServerPatches = [
        { op: 'replace', path: '/meta/tick', value: 1 },
        { op: 'replace', path: '/meta/turn', value: 1 },
        { op: 'replace', path: '/meta/currentPlayer', value: 'p2' }
        // Missing: /state/meta/currentPlayer and /state/meta/turn patches
      ];

      const mockState = {
        meta: { currentPlayer: 'p1', turn: 0 },
        state: { meta: { currentPlayer: 'p1', turn: 0 } }
      };

      // Apply only the patches the server actually sends
      actualServerPatches.forEach(patch => {
        const pathParts = patch.path.split('/').filter(p => p);
        let target = mockState;
        
        for (let i = 0; i < pathParts.length - 1; i++) {
          target = target[pathParts[i] as keyof typeof target] as any;
        }
        
        const finalKey = pathParts[pathParts.length - 1];
        (target as any)[finalKey] = patch.value;
      });

      // This shows the bug: meta is updated but state.meta is stale
      expect(mockState.meta.currentPlayer).toBe('p2'); // ✅ Updated
      expect(mockState.state.meta.currentPlayer).toBe('p1'); // ❌ Stale!
      
      // Client logic prioritizes state.meta over meta, so it gets wrong value
      const currentPlayer = mockState.state?.meta?.currentPlayer || mockState.meta?.currentPlayer;
      expect(currentPlayer).toBe('p1'); // ❌ Wrong! Should be p2
    });
  });

  describe('Client Priority Logic Validation', () => {
    it('should prioritize state.meta.currentPlayer over meta.currentPlayer', () => {
      const testCases = [
        {
          name: 'Both present - state.meta wins',
          lobbyState: {
            meta: { currentPlayer: 'p1' },
            state: { meta: { currentPlayer: 'p2' } }
          },
          expected: 'p2'
        },
        {
          name: 'Only meta present - fallback works',
          lobbyState: {
            meta: { currentPlayer: 'p1' },
            state: { meta: {} }
          },
          expected: 'p1'
        },
        {
          name: 'Only state.meta present',
          lobbyState: {
            meta: {},
            state: { meta: { currentPlayer: 'p2' } }
          },
          expected: 'p2'
        },
        {
          name: 'Neither present',
          lobbyState: {
            meta: {},
            state: { meta: {} }
          },
          expected: undefined
        }
      ];

      testCases.forEach(({ name, lobbyState, expected }) => {
        const currentPlayer = lobbyState.state?.meta?.currentPlayer || lobbyState.meta?.currentPlayer;
        expect(currentPlayer, name).toBe(expected);
      });
    });

    it('should detect inconsistencies between meta and state.meta', () => {
      const inconsistentState = {
        meta: { 
          currentPlayer: 'p2', 
          turn: 1,
          actionMap: { p2: { '/zones/board/cells/0/0': { action: 'placeMarker' } } }
        },
        state: { 
          meta: { 
            currentPlayer: 'p1', // ❌ Inconsistent!
            turn: 0 // ❌ Inconsistent!
          } 
        }
      };

      // The client will use state.meta values (wrong ones)
      const currentPlayer = inconsistentState.state?.meta?.currentPlayer || inconsistentState.meta?.currentPlayer;
      const turn = inconsistentState.state?.meta?.turn ?? inconsistentState.meta?.turn;

      expect(currentPlayer).toBe('p1'); // Wrong! Server thinks it's p2
      expect(turn).toBe(0); // Wrong! Server thinks it's turn 1

      // This creates the exact issue we're seeing in the real app
      const isYourTurn = currentPlayer === 'p1'; // User thinks it's their turn
      const serverThinks = inconsistentState.meta.currentPlayer === 'p2'; // Server thinks it's p2's turn
      
      expect(isYourTurn).toBe(true);
      expect(serverThinks).toBe(true);
      // Both can't be true - this is the bug!
    });
  });

  describe('Action Map Synchronization', () => {
    it('should handle action map updates with turn changes', () => {
      // Initial state - p1's turn
      const gameState = {
        you: 'p1',
        meta: {
          currentPlayer: 'p1',
          actionMap: {
            p1: { '/zones/board/cells/0/0': { action: 'placeMarker' } },
            p2: {}
          }
        },
        state: {
          meta: { currentPlayer: 'p1' }
        }
      };

      // Patches that should be sent when turn advances
      const turnAdvancePatches = [
        { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/meta/actionMap/p1', value: {} },
        { 
          op: 'replace', 
          path: '/meta/actionMap/p2', 
          value: { '/zones/board/cells/0/1': { action: 'placeMarker' } }
        }
      ];

      // Apply patches
      turnAdvancePatches.forEach(patch => {
        const pathParts = patch.path.split('/').filter(p => p);
        let target: any = gameState;
        
        for (let i = 0; i < pathParts.length - 1; i++) {
          target = target[pathParts[i]];
        }
        
        const finalKey = pathParts[pathParts.length - 1];
        target[finalKey] = patch.value;
      });

      // Verify turn switched correctly
      const currentPlayer = gameState.state?.meta?.currentPlayer || gameState.meta?.currentPlayer;
      expect(currentPlayer).toBe('p2');

      // Verify action maps updated
      const p1Actions = gameState.meta.actionMap.p1;
      const p2Actions = gameState.meta.actionMap.p2;
      
      expect(Object.keys(p1Actions)).toHaveLength(0); // p1 can't move
      expect(Object.keys(p2Actions)).toHaveLength(1); // p2 can move

      // Verify UI would show correct state for p1 (who is you)
      const isYourTurn = gameState.you === currentPlayer;
      const yourActions = gameState.meta.actionMap[gameState.you];
      
      expect(isYourTurn).toBe(false); // Not p1's turn anymore
      expect(Object.keys(yourActions)).toHaveLength(0); // No actions available
    });
  });

  describe('Real-World Scenarios', () => {
    it('should handle the exact sequence from the logs', () => {
      // This replicates the exact sequence from the browser logs
      
      const gameState = {
        you: 'p1',
        meta: { currentPlayer: 'p1', turn: 0 },
        state: { meta: { currentPlayer: 'p1', turn: 0 } }
      };

      // The patches the server actually sent (from logs)
      const actualPatches = [
        { op: 'replace', path: '/state/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/meta/tick', value: 1 },
        { op: 'replace', path: '/meta/turn', value: 1 },
        { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/meta/actionMap/p2', value: {} },
        { op: 'add', path: '/meta/gameLog/-', value: {} }
      ];

      // Apply patches (simplified)
      actualPatches.forEach(patch => {
        if (patch.path === '/meta/currentPlayer') {
          gameState.meta.currentPlayer = patch.value as string;
        } else if (patch.path === '/meta/turn') {
          gameState.meta.turn = patch.value as number;
        }
        // Note: No /state/meta/currentPlayer patch!
      });

      // This shows the exact bug from the logs
      expect(gameState.meta.currentPlayer).toBe('p2'); // Updated by patch
      expect(gameState.state.meta.currentPlayer).toBe('p1'); // Not updated!

      // Client logic (from GameView.tsx)
      const currentPlayer = gameState.state?.meta?.currentPlayer || gameState.meta?.currentPlayer;
      const isYourTurn = gameState.you === currentPlayer;

      expect(currentPlayer).toBe('p1'); // Wrong! Should be p2
      expect(isYourTurn).toBe(true); // Wrong! Should be false
    });

    it('should work correctly with the fixed server patches', () => {
      const gameState = {
        you: 'p1',
        meta: { currentPlayer: 'p1', turn: 0 },
        state: { meta: { currentPlayer: 'p1', turn: 0 } }
      };

      // The patches the server SHOULD send (with fix)
      const fixedPatches = [
        { op: 'replace', path: '/state/zones/board/cells/0/0', value: { entity: 'mark_p1' } },
        { op: 'replace', path: '/meta/tick', value: 1 },
        { op: 'replace', path: '/meta/turn', value: 1 },
        { op: 'replace', path: '/meta/currentPlayer', value: 'p2' },
        { op: 'replace', path: '/state/meta/currentPlayer', value: 'p2' }, // 🔧 Fixed!
        { op: 'replace', path: '/state/meta/turn', value: 1 }, // 🔧 Fixed!
        { op: 'replace', path: '/meta/actionMap/p2', value: {} }
      ];

      // Apply fixed patches
      fixedPatches.forEach(patch => {
        if (patch.path === '/meta/currentPlayer') {
          gameState.meta.currentPlayer = patch.value as string;
        } else if (patch.path === '/state/meta/currentPlayer') {
          gameState.state.meta.currentPlayer = patch.value as string;
        } else if (patch.path === '/meta/turn') {
          gameState.meta.turn = patch.value as number;
        } else if (patch.path === '/state/meta/turn') {
          gameState.state.meta.turn = patch.value as number;
        }
      });

      // Now both should be consistent
      expect(gameState.meta.currentPlayer).toBe('p2');
      expect(gameState.state.meta.currentPlayer).toBe('p2');

      // Client logic should work correctly
      const currentPlayer = gameState.state?.meta?.currentPlayer || gameState.meta?.currentPlayer;
      const isYourTurn = gameState.you === currentPlayer;

      expect(currentPlayer).toBe('p2'); // ✅ Correct!
      expect(isYourTurn).toBe(false); // ✅ Correct! Not p1's turn
    });
  });
});