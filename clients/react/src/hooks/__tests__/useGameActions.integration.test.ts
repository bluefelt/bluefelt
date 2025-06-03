import { describe, it, expect, vi } from 'vitest';
import { getPlayerEntity } from '../../utils/entityUtils';

// Simulate the logic from useGameActions for different games
describe('useGameActions Entity Selection Integration', () => {
  
  function simulateEntitySelection(entities: any[], playerStr: string) {
    // This mimics the logic in useGameActions.ts
    const playerNum = parseInt(playerStr.substring(1)); // "p1" -> 1
    const playerEntity = getPlayerEntity(entities, playerNum);
    const entityId = playerEntity?.id || `mark_${playerStr}`;
    return entityId;
  }

  it('should send correct entity for Three Mens Morris game', () => {
    // Simulate Three Men's Morris entity definitions
    const threeMensEntities = [
      {
        id: 'piece_p1',
        props: { owner: 'p1', value: 'p1' },
        quantity: 3,
        ui: { tokenType: 'p1' }
      },
      {
        id: 'piece_p2', 
        props: { owner: 'p2', value: 'p2' },
        quantity: 3,
        ui: { tokenType: 'p2' }
      }
    ];

    const p1EntityId = simulateEntitySelection(threeMensEntities, 'p1');
    const p2EntityId = simulateEntitySelection(threeMensEntities, 'p2');

    // Should use piece_ entities, not mark_
    expect(p1EntityId).toBe('piece_p1');
    expect(p2EntityId).toBe('piece_p2');
  });

  it('should send correct entity for Tic Tac Toe game', () => {
    // Simulate Tic Tac Toe entity definitions
    const ticTacToeEntities = [
      {
        id: 'mark_p1',
        props: { value: 'p1' },
        ui: { tokenType: 'p1' }
      },
      {
        id: 'mark_p2',
        props: { value: 'p2' },
        ui: { tokenType: 'p2' }
      }
    ];

    const p1EntityId = simulateEntitySelection(ticTacToeEntities, 'p1');
    const p2EntityId = simulateEntitySelection(ticTacToeEntities, 'p2');

    // Should use mark_ entities
    expect(p1EntityId).toBe('mark_p1');
    expect(p2EntityId).toBe('mark_p2');
  });

  it('should send correct entity for Connect Four game', () => {
    // Simulate Connect Four entity definitions  
    const connectFourEntities = [
      {
        id: 'disc_p1',
        props: { value: 'p1' },
        ui: { tokenType: 'p1' }
      },
      {
        id: 'disc_p2',
        props: { value: 'p2' },
        ui: { tokenType: 'p2' }
      }
    ];

    const p1EntityId = simulateEntitySelection(connectFourEntities, 'p1');
    const p2EntityId = simulateEntitySelection(connectFourEntities, 'p2');

    // Should use disc_ entities
    expect(p1EntityId).toBe('disc_p1');
    expect(p2EntityId).toBe('disc_p2');
  });

  it('should fallback to mark_ when no entities found', () => {
    // Simulate case where entity lookup fails
    const emptyEntities: any[] = [];

    const p1EntityId = simulateEntitySelection(emptyEntities, 'p1');
    const p2EntityId = simulateEntitySelection(emptyEntities, 'p2');

    // Should fallback to old behavior
    expect(p1EntityId).toBe('mark_p1');
    expect(p2EntityId).toBe('mark_p2');
  });

  it('should handle the exact bug scenario', () => {
    // This simulates the exact bug: Three Men's Morris entities with old hardcoded logic
    const threeMensEntities = [
      {
        id: 'piece_p1',
        props: { owner: 'p1', value: 'p1' },
        quantity: 3,
        ui: { tokenType: 'p1' }
      },
      {
        id: 'piece_p2',
        props: { owner: 'p2', value: 'p2' },
        quantity: 3,
        ui: { tokenType: 'p2' }
      }
    ];

    // Old hardcoded logic (the bug)
    const oldEntityId = `mark_p1`;
    
    // New dynamic logic (the fix)
    const newEntityId = simulateEntitySelection(threeMensEntities, 'p1');
    
    // Verify the fix works
    expect(oldEntityId).toBe('mark_p1'); // This was the problem
    expect(newEntityId).toBe('piece_p1'); // This is the solution
    expect(newEntityId).not.toBe(oldEntityId); // Confirms the fix changes the behavior
  });
});