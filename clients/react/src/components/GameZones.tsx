import Board from './zones/Board';
import CardZone from './CardZone';
import { usePlayer } from '../context/PlayerContext';

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
}

export default function GameZones({
  zones,
  entityDefinitions,
  onCellClick,
  onCardAction,
  isMyTurn = false,
  zoneMetadata,
  playerNames,
  actionMap = {},
  selection,
  you
}: GameZonesProps) {
  const { player } = usePlayer();
  
  if (!zones) return null;

  // Separate zones by type
  const gridZones: any = {};
  const cardZones: any = {};
  
  // Look at zone metadata to determine zone types
  console.log('[GameZones] Processing zones:', {
    zoneKeys: Object.keys(zones),
    zoneMetadataLength: zoneMetadata?.length,
    zoneMetadata: zoneMetadata,
    actionMap: actionMap,
    you: you
  });
  
  Object.entries(zones).forEach(([zoneId, zoneData]) => {
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    console.log(`[GameZones] Processing zone ${zoneId}:`, {
      zoneData,
      zoneMeta,
      shape: zoneMeta?.shape
    });
    
    if (zoneMeta?.shape === 'stack' || zoneMeta?.shape === 'list' || zoneMeta?.shape === 'deck') {
      // This is explicitly a card zone
      // Extract items array if it exists (server sends {items: [...]})
      if (zoneData && typeof zoneData === 'object' && 'items' in zoneData) {
        cardZones[zoneId] = (zoneData as any).items;
      } else if (Array.isArray(zoneData)) {
        cardZones[zoneId] = zoneData;
      } else {
        cardZones[zoneId] = [];
      }
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
    const actions: Array<{ cardIndex: number; verb: string }> = [];
    
    // Check action map for this zone's cards
    Object.entries(actionMap).forEach(([location, action]) => {
      // Parse location like "/zones/hand_p1/2"
      const parts = location.split('/');
      if (parts[2] === zoneId && parts[3]) {
        const cardIndex = parseInt(parts[3]);
        if (!isNaN(cardIndex)) {
          actions.push({
            cardIndex,
            verb: (action as any).action || (action as any).verb
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

  return (
    <div className="w-full">
      {/* Grid zones (boards) */}
      {Object.keys(gridZones).length > 0 && (
        <Board
          zones={gridZones}
          entityDefinitions={entityDefinitions}
          onCellClick={onCellClick}
          isMyTurn={isMyTurn}
          zoneMetadata={zoneMetadata}
          playerNames={playerNames}
          actionMap={actionMap}
          selection={selection}
        />
      )}
      
      {/* Card zones */}
      {Object.entries(cardZones).map(([zoneId, cards]) => {
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
            showCount={zoneMeta?.ui?.showCount || zoneId === 'deck'}
            showTop={zoneMeta?.ui?.showTop}
            onCardClick={(cardId, cardIndex) => {
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
      })}
    </div>
  );
}