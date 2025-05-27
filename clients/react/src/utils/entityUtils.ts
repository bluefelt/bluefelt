import type { EntityDefinition } from '../types/messages';

/**
 * Build a mapping from entity ID to display glyph
 */
export function buildGlyphMapping(entities?: EntityDefinition[]): Record<string, string> {
  if (!entities) return {};
  
  const mapping: Record<string, string> = {};
  
  entities.forEach(entity => {
    if (entity.ui?.glyph) {
      mapping[entity.id] = entity.ui.glyph;
    }
  });
  
  return mapping;
}

/**
 * Get the display glyph for an entity, fallback to the entity ID if no glyph defined
 */
export function getEntityGlyph(entityId: string | null | undefined, glyphMapping: Record<string, string>): string {
  if (!entityId) return '';
  return glyphMapping[entityId] || entityId;
}