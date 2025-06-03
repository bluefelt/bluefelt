import React from 'react';
import Board from './zones/Board';
import CardZone from './CardZone';
import { usePlayer } from '../context/PlayerContext';
import type { ZoneGroup } from '../types/messages';

interface GameZonesProps {
  zones: any;
  entityDefinitions?: any[];
  onCellClick?: (row: number, col: number) => void;
  onCardAction?: (zoneId: string, cardIndex: number) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[];
  playerNames?: string[];
  actionMap?: Record<string, any>;
  selection?: any;
  you?: string;
  zoneGroups?: ZoneGroup[];
}

function GameZones({
  zones,
  entityDefinitions,
  onCellClick,
  onCardAction,
  isMyTurn = false,
  zoneMetadata,
  playerNames,
  actionMap = {},
  selection,
  you,
  zoneGroups
}: GameZonesProps) {
  const { player } = usePlayer();
  
  if (!zones) return null;


  // Separate zones by type
  const gridZones: any = {};
  const cardZones: any = {};
  
  // Look at zone metadata to determine zone types
  
  Object.entries(zones).forEach(([zoneId, zoneData]) => {
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    
    if (zoneMeta?.shape === 'stack' || zoneMeta?.shape === 'list' || zoneMeta?.shape === 'deck' || zoneMeta?.shape === 'single' ||
        zoneMeta?.type === 'stack' || zoneMeta?.type === 'list' || zoneMeta?.type === 'deck' || zoneMeta?.type === 'single') {
      // This is explicitly a card/entity zone
      // Extract items array if it exists (server sends {items: [...]})
      if (zoneData && typeof zoneData === 'object' && 'items' in zoneData) {
        cardZones[zoneId] = (zoneData as any).items;
      } else if (Array.isArray(zoneData)) {
        cardZones[zoneId] = zoneData;
      } else {
        cardZones[zoneId] = [];
      }
    } else if (zoneData && typeof zoneData === 'object' && 'items' in zoneData) {
      // Zone has items structure even without metadata - it's a card zone
      cardZones[zoneId] = (zoneData as any).items || [];
    } else if (zoneData && typeof zoneData === 'object' && 'cells' in zoneData && 'type' in zoneData && (zoneData as any).type === 'grid') {
      // New server format: {cells: [[...]], type: "grid"}
      gridZones[zoneId] = (zoneData as any).cells;
    } else if (Array.isArray(zoneData) && zoneData.length > 0 && Array.isArray(zoneData[0])) {
      // Check if it's a single-row grid that should be treated as a card zone
      // Card zones typically have names like hand_, deck, table, discard
      const isCardZone = zoneData.length === 1 && 
        (zoneId.includes('hand') || zoneId.includes('deck') || 
         zoneId === 'table' || zoneId === 'discard');
      
      if (isCardZone) {
        // Convert single row to flat array for card zone
        cardZones[zoneId] = zoneData[0];
      } else {
        // This is a regular grid zone
        gridZones[zoneId] = zoneData;
      }
    }
  });

  // Get possible actions for cards
  const getCardActions = (zoneId: string) => {
    const actions: Array<{ cardIndex: number; action: string }> = [];
    
    // Check action map for this zone's cards
    Object.entries(actionMap).forEach(([location, action]) => {
      // Parse location like "/zones/hand_p1/2"
      const parts = location.split('/');
      if (parts[2] === zoneId && parts[3]) {
        const cardIndex = parseInt(parts[3]);
        if (!isNaN(cardIndex)) {
          actions.push({
            cardIndex,
            action: (action as any).action || (action as any).verb
          });
        }
      }
    });
    
    return actions;
  };

  // Determine if user owns a zone
  const isZoneOwner = (zoneId: string): boolean => {
    // Check if zone ID contains player indicator
    if (zoneId.includes('_p1') && you === 'p1') return true;
    if (zoneId.includes('_p2') && you === 'p2') return true;
    
    // Check if zone ID contains username
    if (player && zoneId.includes(player.username)) return true;
    
    return false;
  };

  // Helper function to render a card zone
  const renderCardZone = (zoneId: string, cards: any) => {
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    if (!zoneMeta) return null;
    
    // Convert card data to array if needed
    const cardArray = Array.isArray(cards) ? cards : [];
    
    // Check if there's a zone-level action (e.g., drawing from deck)
    const zoneLocation = `/zones/${zoneId}`;
    const hasZoneAction = actionMap[zoneLocation] !== undefined;
    
    return (
      <CardZone
        key={zoneId}
        zoneId={zoneId}
        zoneName={zoneMeta?.name || zoneId}
        cards={cardArray
          .filter((cardId: string) => cardId !== null)
          .map((cardId: string) => {
            const entity = entityDefinitions?.find(e => e.id === cardId);
            return entity || { id: cardId };
          })}
        layout={zoneMeta?.ui?.layout || (zoneId.includes('hand') ? 'fan' : 'spread')}
        visibility={zoneMeta?.visibility || 'all'}
        isOwner={isZoneOwner(zoneId)}
        showCount={zoneMeta?.ui?.showCount !== undefined ? zoneMeta.ui.showCount : (zoneMeta?.ui?.layout === 'stack' || zoneId.includes('deck') || zoneId.includes('Pile'))}
        showTop={zoneMeta?.ui?.showTop}
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
        possibleActions={getCardActions(zoneId)}
        hasZoneAction={hasZoneAction}
        playerNames={playerNames}
        you={you}
        playerColor={player?.color}
      />
    );
  };

  // Organize zones based on groups
  const zonesInGroups = new Set<string>();
  const groupsList = zoneGroups || [];
  
  
  // Reorder groups to put current player's zones at the bottom
  const reorderedGroups = [...groupsList];
  if (you) {
    // Categorize groups
    const currentPlayerGroups: typeof groupsList = [];
    const otherPlayerGroups: typeof groupsList = [];
    const neutralGroups: typeof groupsList = [];
    
    groupsList.forEach(group => {
      // Check if this is a player-specific group by looking at zone IDs
      const hasPlayerZones = group.zones.some(zoneId => 
        zoneId.includes('_p1') || zoneId.includes('_p2')
      );
      
      if (hasPlayerZones) {
        // It's a player group - check if it's current player's
        const isCurrentPlayerGroup = group.zones.some(zoneId => 
          zoneId.includes(`_${you}`)
        );
        
        if (isCurrentPlayerGroup) {
          currentPlayerGroups.push(group);
        } else {
          otherPlayerGroups.push(group);
        }
      } else {
        // Non-player group (like table area)
        neutralGroups.push(group);
      }
    });
    
    // Reorder: current player groups at top, then neutral groups, then other player groups
    reorderedGroups.length = 0;
    reorderedGroups.push(...currentPlayerGroups, ...neutralGroups, ...otherPlayerGroups);
  }
  
  // Track which zones are in groups
  reorderedGroups.forEach(group => {
    group.zones.forEach(zoneId => zonesInGroups.add(zoneId));
  });

  // Get ungrouped zones
  const ungroupedGridZones: any = {};
  const ungroupedCardZones: any = {};
  
  Object.entries(gridZones).forEach(([zoneId, zoneData]) => {
    if (!zonesInGroups.has(zoneId)) {
      ungroupedGridZones[zoneId] = zoneData;
    }
  });
  
  Object.entries(cardZones).forEach(([zoneId, cards]) => {
    if (!zonesInGroups.has(zoneId)) {
      ungroupedCardZones[zoneId] = cards;
    }
  });

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
                return (
                  <Board
                    key={zoneId}
                    zones={{ [zoneId]: gridZones[zoneId] }}
                    entityDefinitions={entityDefinitions}
                    onCellClick={onCellClick}
                    isMyTurn={isMyTurn}
                    zoneMetadata={zoneMetadata}
                    playerNames={playerNames}
                    actionMap={actionMap}
                    selection={selection}
                  />
                );
              } else if (cardZones[zoneId]) {
                // This is a card zone in the group
                return renderCardZone(zoneId, cardZones[zoneId]);
              }
              return null;
            })}
          </div>
        </div>
      ))}

      {/* Render ungrouped grid zones */}
      {Object.keys(ungroupedGridZones).length > 0 && (
        <Board
          zones={ungroupedGridZones}
          entityDefinitions={entityDefinitions}
          onCellClick={onCellClick}
          isMyTurn={isMyTurn}
          zoneMetadata={zoneMetadata}
          playerNames={playerNames}
          actionMap={actionMap}
          selection={selection}
        />
      )}
      
      {/* Render ungrouped card zones */}
      {Object.entries(ungroupedCardZones).map(([zoneId, cards]) => 
        renderCardZone(zoneId, cards)
      )}
    </div>
  );
}

export default React.memo(GameZones);
