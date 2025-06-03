import { describe, it, expect, vi } from 'vitest';
import { getPlayerEntity } from '../../utils/entityUtils';
import type { EntityDefinition } from '../../types/messages';

describe('Entity Lookup for Game Actions', () => {
  it('should correctly identify piece entities for Three Mens Morris', () => {
    const entities: EntityDefinition[] = [
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

    const p1Entity = getPlayerEntity(entities, 1);
    const p2Entity = getPlayerEntity(entities, 2);

    expect(p1Entity?.id).toBe('piece_p1');
    expect(p2Entity?.id).toBe('piece_p2');
  });

  it('should correctly identify mark entities for Tic Tac Toe', () => {
    const entities: EntityDefinition[] = [
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

    const p1Entity = getPlayerEntity(entities, 1);
    const p2Entity = getPlayerEntity(entities, 2);

    expect(p1Entity?.id).toBe('mark_p1');
    expect(p2Entity?.id).toBe('mark_p2');
  });

  it('should correctly identify disc entities for Connect Four', () => {
    const entities: EntityDefinition[] = [
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

    const p1Entity = getPlayerEntity(entities, 1);
    const p2Entity = getPlayerEntity(entities, 2);

    expect(p1Entity?.id).toBe('disc_p1');
    expect(p2Entity?.id).toBe('disc_p2');
  });

  it('should prioritize mark over piece when both exist', () => {
    // Test the priority system in entityUtils
    const entities: EntityDefinition[] = [
      {
        id: 'piece_p1',
        props: { value: 'p1' },
        ui: { tokenType: 'p1' }
      },
      {
        id: 'mark_p1',
        props: { value: 'p1' },
        ui: { tokenType: 'p1' }
      }
    ];

    const p1Entity = getPlayerEntity(entities, 1);

    // mark has higher priority than piece
    expect(p1Entity?.id).toBe('mark_p1');
  });

  it('should return null when no matching entities exist', () => {
    const entities: EntityDefinition[] = [
      {
        id: 'some_other_entity',
        props: { value: 'neutral' },
        ui: { tokenType: 'neutral' }
      }
    ];

    const p1Entity = getPlayerEntity(entities, 1);
    expect(p1Entity).toBe(null);
  });

  it('should handle empty or undefined entity arrays', () => {
    expect(getPlayerEntity([], 1)).toBe(null);
    expect(getPlayerEntity(undefined, 1)).toBe(null);
  });
});