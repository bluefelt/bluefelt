// Utility functions for working with game entities

export interface EntityDefinition {
  id: string;
  ui?: {
    tokenType?: string;
    svg?: string;
    color?: string;
  };
  props?: {
    value?: string;
  };
}

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
  
  // Check for SVG first
  if (entity.ui?.svg) {
    return {
      svg: entity.ui.svg,
      type: 'svg',
      value: entity.props?.value || entity.ui?.tokenType || '',
    };
  }
  
  // Check for token type (X, O, etc.)
  if (entity.ui?.tokenType) {
    return {
      text: entity.ui.tokenType.toUpperCase(),
      type: 'token',
      value: entity.ui.tokenType,
    };
  }
  
  // Check for props value
  if (entity.props?.value) {
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