// Simplified entity utilities - server authority approach
import type { EntityDefinition } from '../types/messages';

/**
 * Get display glyph for an entity using server-provided data
 * @param entityId - The entity ID 
 * @param entities - Server-provided entity definitions
 * @returns The display glyph or entity ID fallback
 */
export function getEntityGlyph(entityId: string | null, entities: EntityDefinition[] | undefined): string {
  if (!entityId || !entities) return entityId || '';
  
  const entity = entities.find(e => e.id === entityId);
  
  // Use server-provided display information in priority order
  if (entity?.ui?.tokenType) {
    return entity.ui.tokenType.toUpperCase();
  }
  if (entity?.ui?.glyph) {
    return entity.ui.glyph.toUpperCase();
  }
  if (entity?.props?.value) {
    return entity.props.value.toUpperCase();
  }
  
  // Fallback to entity ID
  return entityId.toUpperCase();
}

/**
 * Find entity definition by ID - simple lookup using server data
 * @param entityId - The entity ID
 * @param entities - Server-provided entity definitions
 * @returns The entity definition or null
 */
export function findEntityById(entityId: string, entities: EntityDefinition[] | undefined): EntityDefinition | null {
  if (!entities) return null;
  return entities.find(e => e.id === entityId) || null;
}

/**
 * Get entity display name using server-provided data
 * @param entityId - The entity ID
 * @param entities - Server-provided entity definitions
 * @returns Display name for the entity
 */
export function getEntityDisplayName(entityId: string | null, entities: EntityDefinition[] | undefined): string {
  if (!entityId || !entities) return entityId || '';
  
  const entity = entities.find(e => e.id === entityId);
  
  // Use server-provided display information
  if (entity?.ui?.displayName) {
    return entity.ui.displayName;
  }
  if (entity?.ui?.tokenType) {
    return entity.ui.tokenType;
  }
  if (entity?.ui?.glyph) {
    return entity.ui.glyph;
  }
  
  return entityId;
}

/**
 * Get player entity for a given player number
 * @param entities - Server-provided entity definitions
 * @param playerNum - Player number (1-based)
 * @returns Player entity definition or null
 */
export function getPlayerEntity(entities: EntityDefinition[] | undefined, playerNum: number): EntityDefinition | null {
  if (!entities) return null;
  
  // Look for common player entity patterns
  const playerPatterns = [
    `mark_p${playerNum}`,
    `piece_p${playerNum}`,
    `disc_p${playerNum}`,
    `token_p${playerNum}`,
    `player_${playerNum}`,
    `p${playerNum}_mark`,
    `p${playerNum}_piece`,
    `p${playerNum}_disc`,
    `p${playerNum}_token`
  ];
  
  for (const pattern of playerPatterns) {
    const entity = entities.find(e => e.id === pattern);
    if (entity) return entity;
  }
  
  return null;
}

/**
 * Get display information for an entity
 * @param entity - Entity definition
 * @param playerNum - Player number for fallback
 * @returns Display object with type and value/text
 */
export function getEntityDisplay(entity: EntityDefinition | null, playerNum: number): { type: 'token' | 'text'; value?: string; text?: string } {
  if (!entity) {
    // Fallback for player number
    return {
      type: 'text',
      text: `P${playerNum}`
    };
  }
  
  // Check if it's a token type
  if (entity.ui?.tokenType) {
    return {
      type: 'token',
      value: entity.ui.tokenType
    };
  }
  
  // Check for glyph
  if (entity.ui?.glyph) {
    return {
      type: 'text',
      text: entity.ui.glyph
    };
  }
  
  // Check for display name
  if (entity.ui?.displayName) {
    return {
      type: 'text',
      text: entity.ui.displayName
    };
  }
  
  // Check props value
  if (entity.props?.value) {
    return {
      type: 'text',
      text: entity.props.value
    };
  }
  
  // Fallback to entity ID
  return {
    type: 'text',
    text: entity.id
  };
}