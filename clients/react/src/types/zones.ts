/**
 * Zone tier system for hierarchical UI organization
 * Based on spatial and informational hierarchy principles
 */

export enum ZoneTier {
  /** Player's private information - highest priority, always visible */
  HAND = 0,
  
  /** Immediate game area - direct interaction spaces */
  TACTICAL = 1,
  
  /** Broader game context - strategic overview */
  STRATEGIC = 2,
  
  /** Background/atmospheric zones - lowest priority */
  AMBIENT = 3
}

export interface ZoneWithTier {
  id: string;
  tier: ZoneTier;
  priority?: number; // Optional sub-ordering within tier
  layout_order?: number; // Server-provided ordering
  visibility?: 'all' | 'owner' | 'hidden';
  owner?: string | null;
  renderType?: 'grid' | 'card' | 'choice' | 'hex';
  resolved_name?: string;
  name?: string;
}

export interface ZoneTierConfig {
  /** Visual styling for each tier */
  styling: {
    [key in ZoneTier]: {
      containerClass: string;
      spacing: string;
      background: string;
      border: string;
    }
  };
  
  /** Layout behavior for each tier */
  layout: {
    [key in ZoneTier]: {
      columns: number;
      responsive: boolean;
      collapsible: boolean;
    }
  };
}

export const DEFAULT_ZONE_TIER_CONFIG: ZoneTierConfig = {
  styling: {
    [ZoneTier.HAND]: {
      containerClass: 'hand-tier',
      spacing: 'space-y-2',
      background: 'bg-blue-50',
      border: 'border-blue-200 border-2'
    },
    [ZoneTier.TACTICAL]: {
      containerClass: 'tactical-tier', 
      spacing: 'space-y-4',
      background: 'bg-white',
      border: 'border-gray-300 border'
    },
    [ZoneTier.STRATEGIC]: {
      containerClass: 'strategic-tier',
      spacing: 'space-y-6', 
      background: 'bg-gray-50',
      border: 'border-gray-200'
    },
    [ZoneTier.AMBIENT]: {
      containerClass: 'ambient-tier',
      spacing: 'space-y-2',
      background: 'bg-gray-100',
      border: 'border-gray-100'
    }
  },
  layout: {
    [ZoneTier.HAND]: {
      columns: 1,
      responsive: false,
      collapsible: false
    },
    [ZoneTier.TACTICAL]: {
      columns: 2,
      responsive: true, 
      collapsible: false
    },
    [ZoneTier.STRATEGIC]: {
      columns: 3,
      responsive: true,
      collapsible: true
    },
    [ZoneTier.AMBIENT]: {
      columns: 1,
      responsive: false,
      collapsible: true
    }
  }
};

/**
 * Classify a zone into appropriate tier based on zone metadata
 */
export function classifyZoneTier(zoneMetadata: any, you?: string): ZoneTier {
  const zoneId = zoneMetadata.id || '';
  const owner = zoneMetadata.owner;
  const visibility = zoneMetadata.visibility;
  const renderType = zoneMetadata.renderType;
  
  // Hand tier: Player's private zones
  if (owner === you && visibility === 'owner') {
    return ZoneTier.HAND;
  }
  
  // Hand tier: Explicitly hand zones
  if (zoneId.includes('hand') || zoneId.includes('Hand')) {
    return ZoneTier.HAND;
  }
  
  // Tactical tier: Interactive game zones
  if (renderType === 'grid' || renderType === 'hex') {
    return ZoneTier.TACTICAL;
  }
  
  if (zoneId.includes('board') || zoneId.includes('Board') || 
      zoneId.includes('play') || zoneId.includes('game')) {
    return ZoneTier.TACTICAL;
  }
  
  // Strategic tier: Information and overview zones  
  if (renderType === 'choice') {
    return ZoneTier.STRATEGIC;
  }
  
  if (zoneId.includes('score') || zoneId.includes('Score') ||
      zoneId.includes('count') || zoneId.includes('Count') ||
      zoneId.includes('view') || zoneId.includes('View') ||
      zoneId.includes('info') || zoneId.includes('Info')) {
    return ZoneTier.STRATEGIC;
  }
  
  // Ambient tier: Background zones
  if (zoneId.includes('deck') || zoneId.includes('Deck') ||
      zoneId.includes('discard') || zoneId.includes('Discard') ||
      zoneId.includes('pile') || zoneId.includes('Pile')) {
    return ZoneTier.AMBIENT;
  }
  
  // Default: Tactical tier for unknown zones
  return ZoneTier.TACTICAL;
}

/**
 * Sort zones by tier and priority
 */
export function sortZonesByTier(zones: ZoneWithTier[]): ZoneWithTier[] {
  return zones.sort((a, b) => {
    // Primary sort: tier (hand > tactical > strategic > ambient)
    if (a.tier !== b.tier) {
      return a.tier - b.tier;
    }
    
    // Secondary sort: server-provided layout_order
    if (a.layout_order !== undefined && b.layout_order !== undefined) {
      return a.layout_order - b.layout_order;
    }
    
    // Tertiary sort: priority within tier
    if (a.priority !== undefined && b.priority !== undefined) {
      return a.priority - b.priority;
    }
    
    // Final sort: alphabetical by ID
    return a.id.localeCompare(b.id);
  });
}