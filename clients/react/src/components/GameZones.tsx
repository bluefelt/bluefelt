import React from 'react';
import Board from './zones/Board';
import { HexBoard } from './zones';
import CardZone from './CardZone';
import { ChoiceZone } from './zones/ChoiceZone';
import { usePlayer } from '../context/PlayerContext';
import type { ZoneGroup } from '../types/messages';

interface GameZonesProps {
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
  zoneGroups?: ZoneGroup[];
  gameId?: string;
  playerPreferences?: Record<string, any>;
  multiStepState?: any;
  useTierSystem?: boolean; // Enable the new zone tier system
}

function GameZones({
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
  zoneGroups,
  gameId,
  playerPreferences,
  multiStepState,
  useTierSystem = false // Temporarily disable tier system due to import issue
}: GameZonesProps) {
  const { player } = usePlayer();
  
  // Use new tier system if enabled and metadata is available
  // Temporarily disabled due to import issue
  // if (useTierSystem && zoneMetadata) {
  //   return (
  //     <TieredZoneRenderer
  //       zones={zones}
  //       entityDefinitions={entityDefinitions}
  //       onCellClick={onCellClick}
  //       onHexClick={onHexClick}
  //       onCardAction={onCardAction}
  //       onChoiceSelect={onChoiceSelect}
  //       isMyTurn={isMyTurn}
  //       zoneMetadata={zoneMetadata}
  //       playerNames={playerNames}
  //       actionMap={actionMap}
  //       selection={selection}
  //       you={you}
  //       playerPreferences={playerPreferences}
  //       multiStepState={multiStepState}
  //     />
  //   );
  // }
  
  if (!zones) return null;

  // Helper function to render board zones
  const renderBoard = (boardZones: any, key?: string) => {
    return (
      <Board
        key={key}
        zones={boardZones}
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
  };

  // Helper function to render choice zones using server authority
  const renderChoiceZone = (zoneId: string, zoneData: any) => {
    console.log(`[GameZones.renderChoiceZone] Rendering ${zoneId}`, {
      zoneData,
      zoneMetadata: zoneMetadata?.find(z => z.id === zoneId),
      you,
      multiStepState
    });
    
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    if (!zoneMeta) {
      console.log(`[GameZones.renderChoiceZone] No metadata for ${zoneId}, skipping`);
      return null;
    }
    
    // Check visibility based on server authority
    const isVisible = zoneMeta.visibility === 'all' || 
                     (zoneMeta.visibility === 'owner' && zoneMeta.owner === you);
    
    if (!isVisible) {
      console.log(`[GameZones.renderChoiceZone] ${zoneId} not visible to ${you}`);
      return null;
    }
    
    // Use server-provided choice data (no more action map parsing!)
    let items = zoneData?.items || [];
    const prompt = zoneData?.prompt || zoneMeta?.resolved_name || 'Make a choice';
    const multiSelect = zoneData?.multiSelect || false;
    
    console.log(`[GameZones.renderChoiceZone] Initial items for ${zoneId}:`, items);
    
    // If we're in a multi-step state and this is a choice zone, check stepActionMap for dynamic choices
    if (multiStepState && items.length === 0) {
      const stepActionMap = multiStepState.stepActionMap || {};
      
      if (multiStepState.actionType === 'bf.selectChoice') {
        console.log('[GameZones.renderChoiceZone] Multi-step choice zone detected, extracting from stepActionMap');
        console.log('[GameZones.renderChoiceZone] stepActionMap:', stepActionMap);
        // Convert stepActionMap entries to choice items
        items = Object.entries(stepActionMap)
          .filter(([path]) => path.startsWith('/ranks/'))
          .map(([path, action]: [string, any]) => ({
            id: action.args?.choice || path.split('/').pop() || '',
            label: action.args?.label || action.args?.choice || path.split('/').pop() || ''
          }));
        console.log('[GameZones.renderChoiceZone] Converted to choice items:', items);
      } else if (multiStepState.actionType === 'bf.selectPlayer') {
        console.log('[GameZones.renderChoiceZone] Multi-step player selection detected');
        console.log('[GameZones.renderChoiceZone] stepActionMap:', stepActionMap);
        // Convert player selection entries to choice items
        items = Object.entries(stepActionMap)
          .filter(([path]) => path.startsWith('/players/'))
          .map(([path, action]: [string, any]) => ({
            id: action.args?.player || path.split('/').pop() || '',
            label: action.args?.label || action.args?.player || path.split('/').pop() || ''
          }));
        console.log('[GameZones.renderChoiceZone] Converted player choices:', items);
      }
    }
    
    // Don't render if no items available
    if (items.length === 0) {
      console.log(`[GameZones.renderChoiceZone] No items for ${zoneId}, returning null`);
      return null;
    }
    
    return (
      <ChoiceZone
        key={zoneId}
        zone={{
          id: zoneId,
          type: 'choice',
          items: items,
          prompt: prompt,
          visibility: zoneMeta.visibility,
          multiSelect: multiSelect
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
  };

  // Separate zones by type using server authority
  const gridZones: any = {};
  const hexGridZones: any = {};
  const cardZones: any = {};
  const choiceZones: any = {};
  
  // Process zones using server-provided metadata with explicit renderType
  if (zoneMetadata && Array.isArray(zoneMetadata)) {
    console.log('[GameZones] Processing zone metadata:', zoneMetadata);
    zoneMetadata.forEach((zoneMeta: any) => {
      // Skip hidden zones
      if (zoneMeta.visibility === 'hidden') {
        return;
      }
      
      const zoneId = zoneMeta.id;
      const zoneData = zones[zoneId];
      
      console.log(`[GameZones] Processing ${zoneId}:`, {
        renderType: zoneMeta.renderType,
        visibility: zoneMeta.visibility,
        owner: zoneMeta.owner,
        zoneData
      });
      
      // Use server authority - check explicit renderType
      switch (zoneMeta.renderType) {
        case 'choice':
          // Use actual zone data, not metadata
          choiceZones[zoneId] = {
            items: zoneData?.items || [],
            prompt: zoneData?.prompt || zoneMeta.prompt || 'Make a choice',
            multiSelect: zoneData?.multiSelect || zoneMeta.multi_select
          };
          console.log(`[GameZones] Added choice zone ${zoneId}:`, choiceZones[zoneId]);
          break;
          
        case 'card':
        case 'stack': // Handle stack zones as card zones
        case 'deck':  // Handle deck zones as card zones
        case 'list':  // Handle list zones as card zones
          // Server provides complete card data with visibility
          cardZones[zoneId] = zoneMeta.cards || [];
          break;
          
        case 'grid':
          // Server provides grid data in the zones object, not metadata
          console.log(`[GameZones] Processing grid zone ${zoneId} with data:`, zoneData);
          gridZones[zoneId] = zoneData || [];
          break;
          
        case 'hex':
          // Server provides hex grid data
          hexGridZones[zoneId] = {
            cells: zoneMeta.cells || {},
            layout: zoneMeta.layout || 'flat',
            radius: zoneMeta.radius || 3
          };
          break;
          
        default:
          // Unknown render type - skip this zone (server should provide valid renderType)
          console.warn(`Unknown renderType '${zoneMeta.renderType}' for zone ${zoneId}`);
          break;
      }
    });
  } else {
    // Fallback: analyze zone data structure when server metadata is missing
    console.warn('No zone metadata provided by server - using fallback zone detection');
    
    Object.entries(zones || {}).forEach(([zoneId, zoneData]: [string, any]) => {
      if (zoneData && typeof zoneData === 'object') {
        // Detect grid zones (have cells array)
        if (Array.isArray(zoneData.cells)) {
          gridZones[zoneId] = {
            cells: zoneData.cells,
            rows: zoneData.cells.length,
            cols: zoneData.cells[0]?.length || 0
          };
        }
        // Detect card zones (have array data with card-like objects)
        else if (Array.isArray(zoneData) && zoneData.some(item => 
          item && typeof item === 'object' && ('suit' in item || 'rank' in item || 'entity' in item)
        )) {
          cardZones[zoneId] = zoneData;
        }
        // Detect choice zones (have items array or similar structure)
        else if (Array.isArray(zoneData) && zoneData.some(item => 
          item && typeof item === 'object' && ('id' in item || 'value' in item)
        )) {
          choiceZones[zoneId] = { items: zoneData };
        }
        // Default to treating arrays as card zones
        else if (Array.isArray(zoneData)) {
          cardZones[zoneId] = zoneData;
        }
      }
    });
  }



  // Helper function to render a card zone using server authority
  const renderCardZone = (zoneId: string, cards: any) => {
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    if (!zoneMeta) return null;
    
    // Check visibility based on server authority
    const isVisible = zoneMeta.visibility === 'all' || 
                     (zoneMeta.visibility === 'owner' && zoneMeta.owner === you);
    
    if (!isVisible) {
      return null;
    }
    
    // Use server-provided card data with visibility information
    const cardArray = Array.isArray(cards) ? cards : [];
    
    // Check if there's a zone-level action (e.g., drawing from deck)
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
            // Handle server-provided card render data
            if (card.entity) {
              const entity = entityDefinitions?.find(e => e.id === card.entity);
              return {
                ...entity || { id: card.entity },
                visible: card.visible,
                interactionHint: card.interaction_hint,
                highlight: card.highlight
              };
            }
            // Fallback for legacy card data
            const cardId = typeof card === 'string' ? card : card?.entity;
            const entity = entityDefinitions?.find(e => e.id === cardId);
            return entity || { id: cardId };
          })}
        layout={zoneMeta?.layout || (zoneId.includes('hand') ? 'fan' : 'spread')}
        visibility={zoneMeta.visibility}
        isOwner={zoneMeta.owner === you}
        showCount={zoneMeta?.show_count !== undefined ? zoneMeta.show_count : (zoneMeta?.layout === 'stack' || zoneId.includes('deck') || zoneId.includes('Pile'))}
        showTop={zoneMeta?.show_top}
        onCardClick={(_cardId, cardIndex) => {
          if (onCardAction) {
            onCardAction(zoneId, cardIndex);
          }
        }}
        onZoneClick={() => {
          if (onCardAction && hasZoneAction) {
            // For zone-level actions, pass -1 as the index
            onCardAction(zoneId, -1);
          }
        }}
        possibleActions={[]}
        hasZoneAction={hasZoneAction}
        playerNames={playerNames}
        you={you}
        playerColor={player?.color}
        playerPreferences={playerPreferences}
        multiStepState={multiStepState}
        actionMap={actionMap}
      />
    );
  };

  // Organize zones using server authority
  const zonesInGroups = new Set<string>();
  const groupsList = zoneGroups || [];
  
  // Server provides layout_order for optimal zone ordering per player
  // Sort zones by server-provided layout_order when available
  const sortedZoneMetadata = zoneMetadata ? 
    [...zoneMetadata].sort((a, b) => (a.layout_order || 0) - (b.layout_order || 0)) : 
    [];
  
  // Reorder groups to put current player's zones at the bottom (legacy behavior for groups)
  const reorderedGroups = [...groupsList];
  if (you) {
    // Categorize groups
    const currentPlayerGroups: typeof groupsList = [];
    const otherPlayerGroups: typeof groupsList = [];
    const neutralGroups: typeof groupsList = [];
    
    groupsList.forEach(group => {
      // Use server-provided zone metadata to determine ownership
      const hasPlayerZones = group.zones.some(zoneId => {
        const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
        return zoneMeta?.owner !== undefined && zoneMeta?.owner !== null;
      });
      
      if (hasPlayerZones) {
        // It's a player group - check if it belongs to current player
        const isCurrentPlayerGroup = group.zones.some(zoneId => {
          const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
          return zoneMeta?.owner === you;
        });
        
        if (isCurrentPlayerGroup) {
          currentPlayerGroups.push(group);
        } else {
          otherPlayerGroups.push(group);
        }
      } else {
        // Non-player group (neutral zones)
        neutralGroups.push(group);
      }
    });
    
    // Reorder: current player groups at top, then neutral groups, then other player groups
    reorderedGroups.length = 0;
    reorderedGroups.push(...currentPlayerGroups, ...neutralGroups, ...otherPlayerGroups);
  }
  
  // Track which zones are in groups
  reorderedGroups.forEach(group => {
    group.zones.forEach(zoneId => {
      // Don't add choice zones to the "in groups" set - we want them to always render as ungrouped
      if (!choiceZones.hasOwnProperty(zoneId)) {
        zonesInGroups.add(zoneId);
      }
    });
  });

  // Legacy ungrouped zone handling for fallback (when no server metadata)
  const ungroupedCardZones: any = {};
  const ungroupedChoiceZones: any = {};
  
  if (!zoneMetadata) {
    Object.entries(cardZones).forEach(([zoneId, cards]) => {
      if (!zonesInGroups.has(zoneId)) {
        ungroupedCardZones[zoneId] = cards;
      }
    });
    
    Object.entries(choiceZones).forEach(([zoneId, zoneData]) => {
      if (!zonesInGroups.has(zoneId)) {
        ungroupedChoiceZones[zoneId] = zoneData;
      }
    });
  }
  
  

  return (
    <div className="w-full">
      {/* Render zone groups first */}
      {reorderedGroups.map((group) => (
        <div key={group.id} className="mb-6">
          <h3 className="text-lg font-semibold mb-3">{group.title}</h3>
          <div className="space-y-4">
            {group.zones.map(zoneId => {
              if (gridZones[zoneId]) {
                // This is a grid zone in the group
                return renderBoard({ [zoneId]: gridZones[zoneId] }, zoneId);
              } else if (hexGridZones[zoneId]) {
                // This is a hex grid zone in the group
                const hexZoneData = hexGridZones[zoneId];
                return (
                  <HexBoard
                    key={zoneId}
                    zoneId={zoneId}
                    hexData={hexZoneData.cells}
                    layout={hexZoneData.layout || 'flat'}
                    isMyTurn={isMyTurn}
                    onHexClick={onHexClick}
                    entityDefinitions={entityDefinitions}
                    zoneMetadata={zoneMetadata}
                    playerNames={playerNames}
                    actionMap={actionMap}
                    selection={selection}
                  />
                );
              } else if (cardZones[zoneId]) {
                // This is a card zone in the group (includes stack, deck, list)
                return renderCardZone(zoneId, cardZones[zoneId]);
              } else if (choiceZones[zoneId]) {
                // This is a choice zone in the group
                return renderChoiceZone(zoneId, choiceZones[zoneId]);
              }
              return null;
            })}
          </div>
        </div>
      ))}

      
      {/* Render ungrouped zones using server-provided layout order */}
      {sortedZoneMetadata
        .filter(zoneMeta => !zonesInGroups.has(zoneMeta.id))
        .map(zoneMeta => {
          const zoneId = zoneMeta.id;
          
          if ((zoneMeta.renderType === 'card' || zoneMeta.renderType === 'stack' || 
               zoneMeta.renderType === 'deck' || zoneMeta.renderType === 'list') && cardZones[zoneId]) {
            return renderCardZone(zoneId, cardZones[zoneId]);
          } else if (zoneMeta.renderType === 'choice' && choiceZones[zoneId]) {
            return renderChoiceZone(zoneId, choiceZones[zoneId]);
          } else if (zoneMeta.renderType === 'grid' && gridZones[zoneId]) {
            return renderBoard({ [zoneId]: gridZones[zoneId] }, zoneId);
          } else if (zoneMeta.renderType === 'hex' && hexGridZones[zoneId]) {
            const hexZoneData = hexGridZones[zoneId];
            return (
              <HexBoard
                key={zoneId}
                zoneId={zoneId}
                hexData={hexZoneData.cells}
                layout={hexZoneData.layout || 'flat'}
                isMyTurn={isMyTurn}
                onHexClick={onHexClick}
                entityDefinitions={entityDefinitions}
                zoneMetadata={zoneMetadata}
                playerNames={playerNames}
                actionMap={actionMap}
                selection={selection}
              />
            );
          }
          return null;
        })}
      
      {/* Fallback: Render any remaining ungrouped zones using legacy logic */}
      {!zoneMetadata && Object.entries(ungroupedCardZones).map(([zoneId, cards]) => 
        renderCardZone(zoneId, cards)
      )}
      
      {!zoneMetadata && Object.entries(ungroupedChoiceZones).map(([zoneId, zoneData]) => 
        renderChoiceZone(zoneId, zoneData)
      )}
      
    </div>
  );
}

export default React.memo(GameZones);
