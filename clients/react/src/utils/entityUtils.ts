// Utility functions for working with game entities
import type { EntityDefinition } from '../types/messages';

// Priority order for entity types (higher priority first)
const ENTITY_PRIORITY = [
  'mark',    // tic-tac-toe
  'chip',    // connect-4
  'disc',    // reversi
  'stone',   // gomoku
  'piece',   // checkers (not king)
  'token',   // generic
];

/**
 * Find the representative entity for a player
 * @param entities - Array of entity definitions
 * @param playerNum - Player number (1, 2, 3, etc.)
 * @returns The entity definition or null
 */
export function getPlayerEntity(entities: EntityDefinition[] | undefined, playerNum: number): EntityDefinition | null {
  if (!entities) return null;
  
  // Find all entities that match this player
  const playerPattern = `_p${playerNum}`;
  const playerEntities = entities.filter(e => e.id.endsWith(playerPattern));
  
  if (playerEntities.length === 0) return null;
  if (playerEntities.length === 1) return playerEntities[0];
  
  // Multiple entities found - use priority system
  for (const priorityType of ENTITY_PRIORITY) {
    const entity = playerEntities.find(e => e.id.startsWith(priorityType));
    if (entity) return entity;
  }
  
  // No priority match found - return first one
  return playerEntities[0];
}

/**
 * Get display representation for a player entity
 * @param entity - The entity definition
 * @param playerNum - Player number for fallback
 * @returns Object with display information
 */
export function getEntityDisplay(entity: EntityDefinition | null, playerNum: number) {
  if (!entity) {
    // Fallback when no entity found
    return {
      text: playerNum.toString(),
      type: 'text',
      value: playerNum.toString(),
    };
  }
  
  // Check for tokenType first (direct from server)
  if (entity.ui?.tokenType) {
    return {
      text: entity.ui.tokenType.toUpperCase(),
      type: 'token',
      value: entity.ui.tokenType.toLowerCase(),
    };
  }
  
  // Check for glyph
  if (entity.ui?.glyph) {
    // Check if this is a token type (x, o, etc.)
    const tokenTypes = ['x', 'o', 'circle'];
    if (tokenTypes.includes(entity.ui.glyph.toLowerCase())) {
      return {
        text: entity.ui.glyph.toUpperCase(),
        type: 'token',
        value: entity.ui.glyph.toLowerCase(),
      };
    }
    return {
      text: entity.ui.glyph.toUpperCase(),
      type: 'glyph',
      value: entity.ui.glyph,
    };
  }
  
  // Check for props value
  if (entity.props?.value) {
    // Check if this is a token type (x, o, etc.)
    const tokenTypes = ['x', 'o', 'circle'];
    if (tokenTypes.includes(entity.props.value.toLowerCase())) {
      return {
        text: entity.props.value.toUpperCase(),
        type: 'token',
        value: entity.props.value.toLowerCase(),
      };
    }
    return {
      text: entity.props.value.toUpperCase(),
      type: 'value',
      value: entity.props.value,
    };
  }
  
  // Last resort - use player number
  return {
    text: playerNum.toString(),
    type: 'text',
    value: playerNum.toString(),
  };
}

/**
 * Get all entities for a specific player
 * @param entities - Array of entity definitions
 * @param playerNum - Player number
 * @returns Array of entities belonging to this player
 */
export function getAllPlayerEntities(entities: EntityDefinition[] | undefined, playerNum: number): EntityDefinition[] {
  if (!entities) return [];
  const playerPattern = `_p${playerNum}`;
  return entities.filter(e => e.id.endsWith(playerPattern));
}

/**
 * Build a mapping from entity IDs to their display glyphs
 * @param entities - Array of entity definitions
 * @returns Map of entity ID to display glyph
 */
export function buildGlyphMapping(entities: EntityDefinition[] | undefined): Map<string, string> {
  const mapping = new Map<string, string>();
  if (!entities) return mapping;
  
  entities.forEach(entity => {
    if (entity.ui?.tokenType) {
      mapping.set(entity.id, entity.ui.tokenType.toUpperCase());
    } else if (entity.ui?.glyph) {
      mapping.set(entity.id, entity.ui.glyph.toUpperCase());
    } else if (entity.props?.value) {
      mapping.set(entity.id, entity.props.value.toUpperCase());
    }
  });
  
  return mapping;
}

/**
 * Get the display glyph for an entity
 * @param entityId - The entity ID (or null)
 * @param glyphMapping - Map of entity IDs to glyphs
 * @returns The display glyph or empty string
 */
export function getEntityGlyph(entityId: string | null, glyphMapping: Map<string, string>): string {
  if (!entityId) return '';
  return glyphMapping.get(entityId) || '';
}