import { useMemo } from 'react';
import { ZoneTier, ZoneWithTier, classifyZoneTier, sortZonesByTier, DEFAULT_ZONE_TIER_CONFIG } from '../types/zones';

interface UseZoneTiersProps {
  zoneMetadata?: any[];
  you?: string;
}

interface ZonesByTier {
  [ZoneTier.HAND]: ZoneWithTier[];
  [ZoneTier.TACTICAL]: ZoneWithTier[];
  [ZoneTier.STRATEGIC]: ZoneWithTier[];
  [ZoneTier.AMBIENT]: ZoneWithTier[];
}

export function useZoneTiers({ zoneMetadata, you }: UseZoneTiersProps) {
  const { zonesByTier, sortedZones } = useMemo(() => {
    if (!zoneMetadata) {
      return {
        zonesByTier: {
          [ZoneTier.HAND]: [],
          [ZoneTier.TACTICAL]: [],
          [ZoneTier.STRATEGIC]: [],
          [ZoneTier.AMBIENT]: []
        },
        sortedZones: []
      };
    }

    // Classify and enrich zone metadata with tier information
    const zonesWithTiers: ZoneWithTier[] = zoneMetadata.map(zoneMeta => {
      const tier = classifyZoneTier(zoneMeta, you);
      
      return {
        id: zoneMeta.id,
        tier,
        layout_order: zoneMeta.layout_order,
        visibility: zoneMeta.visibility,
        owner: zoneMeta.owner,
        renderType: zoneMeta.renderType,
        resolved_name: zoneMeta.resolved_name,
        name: zoneMeta.name,
        // Assign priority based on zone characteristics
        priority: calculateZonePriority(zoneMeta, you)
      };
    });

    // Sort zones by tier hierarchy
    const sorted = sortZonesByTier(zonesWithTiers);

    // Group zones by tier
    const byTier: ZonesByTier = {
      [ZoneTier.HAND]: [],
      [ZoneTier.TACTICAL]: [],
      [ZoneTier.STRATEGIC]: [],
      [ZoneTier.AMBIENT]: []
    };

    sorted.forEach(zone => {
      byTier[zone.tier].push(zone);
    });

    return {
      zonesByTier: byTier,
      sortedZones: sorted
    };
  }, [zoneMetadata, you]);

  const config = DEFAULT_ZONE_TIER_CONFIG;

  return {
    zonesByTier,
    sortedZones,
    config,
    
    // Utility functions
    getZonesTier: (zoneId: string): ZoneTier | undefined => {
      return sortedZones.find(z => z.id === zoneId)?.tier;
    },
    
    getTierZones: (tier: ZoneTier): ZoneWithTier[] => {
      return zonesByTier[tier];
    },
    
    isZoneInTier: (zoneId: string, tier: ZoneTier): boolean => {
      return zonesByTier[tier].some(z => z.id === zoneId);
    }
  };
}

/**
 * Calculate priority within tier based on zone characteristics
 */
function calculateZonePriority(zoneMeta: any, you?: string): number {
  let priority = 0;
  
  // Higher priority for current player's zones
  if (zoneMeta.owner === you) {
    priority -= 10;
  }
  
  // Higher priority for interactive zones
  if (zoneMeta.renderType === 'grid' || zoneMeta.renderType === 'hex') {
    priority -= 5;
  }
  
  // Lower priority for information-only zones
  if (zoneMeta.renderType === 'choice') {
    priority += 5;
  }
  
  // Use server layout_order as base if available
  if (zoneMeta.layout_order !== undefined) {
    priority += zoneMeta.layout_order;
  }
  
  return priority;
}