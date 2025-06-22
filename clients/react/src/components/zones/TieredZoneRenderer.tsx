import React from 'react';
import { ZoneTier, ZoneWithTier } from '../../types/zones';
import { useZoneTiers } from '../../hooks/useZoneTiers';
import Board from './Board';
import { HexBoard } from './index';
import CardZone from '../CardZone';
import { ChoiceZone } from './ChoiceZone';
import { ViewZone } from './ViewZone';

interface TieredZoneRendererProps {
  zones: any;
  entityDefinitions?: any[];
  onCellClick?: (row: number, col: number) => void;
  onHexClick?: (q: number, r: number) => void;
  onCardAction?: (zoneId: string, cardIndex: number) => void;
  onChoiceSelect?: (zoneId: string, choice: string) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[];
  playerNames?: string[];
  actionMap?: Record<string, any>;
  selection?: any;
  you?: string;
  playerPreferences?: Record<string, any>;
  multiStepState?: any;
}

export function TieredZoneRenderer({
  zones,
  entityDefinitions,
  onCellClick,
  onHexClick,
  onCardAction,
  onChoiceSelect,
  isMyTurn = false,
  zoneMetadata,
  playerNames,
  actionMap = {},
  selection,
  you,
  playerPreferences,
  multiStepState
}: TieredZoneRendererProps) {
  const { zonesByTier, config } = useZoneTiers({ zoneMetadata, you });

  const renderZone = (zoneWithTier: ZoneWithTier): React.ReactNode => {
    const { id: zoneId, renderType, visibility, owner } = zoneWithTier;
    const zoneData = zones[zoneId];
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);

    // Check visibility
    if (visibility === 'hidden') {
      return null;
    }

    if (visibility === 'owner' && owner !== you) {
      return null;
    }

    switch (renderType) {
      case 'grid':
        return (
          <Board
            key={zoneId}
            zones={{ [zoneId]: zoneData }}
            entityDefinitions={entityDefinitions}
            onCellClick={onCellClick}
            isMyTurn={isMyTurn}
            zoneMetadata={zoneMetadata}
            playerNames={playerNames}
            actionMap={actionMap}
            currentPlayerActionMap={actionMap}
            selection={selection}
            playerPreferences={playerPreferences}
            multiStepState={multiStepState}
          />
        );

      case 'hex':
        const hexZoneData = {
          cells: zoneMeta?.cells || {},
          layout: zoneMeta?.layout || 'flat',
          radius: zoneMeta?.radius || 3
        };
        return (
          <HexBoard
            key={zoneId}
            zoneId={zoneId}
            hexData={hexZoneData.cells}
            layout={hexZoneData.layout}
            isMyTurn={isMyTurn}
            onHexClick={onHexClick}
            entityDefinitions={entityDefinitions}
            zoneMetadata={zoneMetadata}
            playerNames={playerNames}
            actionMap={actionMap}
            selection={selection}
          />
        );

      case 'card':
      case 'stack': // Handle stack zones as card zones
      case 'deck':  // Handle deck zones as card zones  
      case 'list':  // Handle list zones as card zones
        const cardArray = Array.isArray(zoneMeta?.cards) ? zoneMeta.cards : 
                         Array.isArray(zoneData) ? zoneData : [];
        
        // Check if there's a zone-level action
        const zoneLocation = `/zones/${zoneId}`;
        const hasZoneAction = actionMap[zoneLocation] !== undefined;

        return (
          <CardZone
            key={zoneId}
            zoneId={zoneId}
            zoneName={zoneMeta?.resolved_name || zoneMeta?.name || zoneId}
            cards={cardArray
              .filter((card: any) => card !== null && card.visible !== false)
              .map((card: any) => {
                if (card.entity) {
                  const entity = entityDefinitions?.find(e => e.id === card.entity);
                  return {
                    ...entity || { id: card.entity },
                    visible: card.visible,
                    interactionHint: card.interaction_hint,
                    highlight: card.highlight
                  };
                }
                const cardId = typeof card === 'string' ? card : card?.entity;
                const entity = entityDefinitions?.find(e => e.id === cardId);
                return entity || { id: cardId };
              })}
            layout={zoneMeta?.layout || (zoneId.includes('hand') ? 'fan' : 'spread')}
            visibility={zoneMeta?.visibility}
            isOwner={zoneMeta?.owner === you}
            showCount={zoneMeta?.show_count !== undefined ? zoneMeta.show_count : 
                      (zoneMeta?.layout === 'stack' || zoneId.includes('deck') || zoneId.includes('Pile'))}
            showTop={zoneMeta?.show_top}
            onCardClick={(_cardId, cardIndex) => {
              if (onCardAction) {
                onCardAction(zoneId, cardIndex);
              }
            }}
            onZoneClick={() => {
              if (onCardAction && hasZoneAction) {
                onCardAction(zoneId, -1);
              }
            }}
            possibleActions={[]}
            hasZoneAction={hasZoneAction}
            playerNames={playerNames}
            you={you}
            playerPreferences={playerPreferences}
            multiStepState={multiStepState}
            actionMap={actionMap}
          />
        );

      case 'choice':
        const items = zoneData?.items || zoneMeta?.items || [];
        const prompt = zoneData?.prompt || zoneMeta?.prompt || zoneMeta?.resolved_name || 'Make a choice';
        const multiSelect = zoneData?.multiSelect || zoneMeta?.multi_select || false;

        if (items.length === 0) {
          return null;
        }

        return (
          <ChoiceZone
            key={zoneId}
            zone={{
              id: zoneId,
              type: 'choice',
              items,
              prompt,
              visibility: zoneMeta?.visibility,
              multiSelect
            }}
            onSelect={(choice: string) => {
              if (onChoiceSelect) {
                onChoiceSelect(zoneId, choice);
              }
            }}
            isActive={isMyTurn}
            className="mb-4"
            multiStepState={multiStepState}
          />
        );

      case 'view':
        const viewData = zoneMeta?.view || {};
        return (
          <ViewZone
            key={zoneId}
            zoneId={zoneId}
            zoneName={zoneMeta?.resolved_name || zoneMeta?.name || zoneId}
            viewType={viewData.view_type || 'strategic'}
            data={viewData.data || {}}
            format={viewData.format}
            playerNames={playerNames}
            you={you}
            className="mb-4"
          />
        );

      default:
        console.warn(`Unknown renderType '${renderType}' for zone ${zoneId}`);
        return null;
    }
  };

  const renderTier = (tier: ZoneTier, zones: ZoneWithTier[]): React.ReactNode => {
    if (zones.length === 0) return null;

    const tierConfig = config.styling[tier];
    const layoutConfig = config.layout[tier];
    
    const tierName = ZoneTier[tier].toLowerCase();
    
    return (
      <div 
        key={tier}
        className={`zone-tier zone-tier-${tierName} ${tierConfig.containerClass} p-4 rounded-lg ${tierConfig.background} ${tierConfig.border}`}
        data-tier={tierName}
      >
        <div className={`${tierConfig.spacing}`}>
          {zones.map(renderZone)}
        </div>
      </div>
    );
  };

  // Fallback for when no metadata is provided
  if (!zoneMetadata) {
    return (
      <div className="w-full">
        <div className="text-sm text-gray-500 mb-4">
          No zone metadata - using fallback rendering
        </div>
        {/* Render zones without tier organization as fallback */}
        {Object.entries(zones || {}).map(([zoneId, zoneData]) => {
          // Simple fallback rendering logic here
          return <div key={zoneId}>{zoneId}: {JSON.stringify(zoneData)}</div>;
        })}
      </div>
    );
  }

  return (
    <div className="w-full tiered-zone-container">
      {/* Render each tier in order: Hand, Tactical, Strategic, Ambient */}
      {renderTier(ZoneTier.HAND, zonesByTier[ZoneTier.HAND])}
      {renderTier(ZoneTier.TACTICAL, zonesByTier[ZoneTier.TACTICAL])}
      {renderTier(ZoneTier.STRATEGIC, zonesByTier[ZoneTier.STRATEGIC])}
      {renderTier(ZoneTier.AMBIENT, zonesByTier[ZoneTier.AMBIENT])}
    </div>
  );
}