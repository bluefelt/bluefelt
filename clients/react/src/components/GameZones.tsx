import Board from './zones/Board';
import CardZone from './CardZone';
import { usePlayer } from '../context/PlayerContext';

interface GameZonesProps {
  zones: any;
  entityDefinitions?: any[];
  onCellClick?: (row: number, col: number) => void;
  onCardAction?: (zoneId: string, cardId: string) => void;
  isMyTurn?: boolean;
  zoneMetadata?: any[];
  playerNames?: string[];
  possibleVerbs?: any[];
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
  possibleVerbs = [],
  selection,
  you
}: GameZonesProps) {
  const { player } = usePlayer();
  
  if (!zones) return null;

  // Separate zones by type
  const gridZones: any = {};
  const cardZones: any = {};
  
  // Look at zone metadata to determine zone types
  Object.entries(zones).forEach(([zoneId, zoneData]) => {
    const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
    
    if (zoneMeta?.shape === 'stack' || zoneMeta?.shape === 'list') {
      // This is explicitly a card zone
      cardZones[zoneId] = zoneData;
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
    const actions: Array<{ cardId: string; verb: string }> = [];
    
    possibleVerbs.forEach(verb => {
      verb.validOptions?.forEach((option: any) => {
        if (option.zone === zoneId && option.entity) {
          actions.push({
            cardId: option.entity,
            verb: verb.verb
          });
        }
      });
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
          possibleVerbs={possibleVerbs}
          selection={selection}
        />
      )}
      
      {/* Card zones */}
      {Object.entries(cardZones).map(([zoneId, cards]) => {
        const zoneMeta = zoneMetadata?.find(z => z.id === zoneId);
        if (!zoneMeta) return null;
        
        // Convert card data to array if needed
        const cardArray = Array.isArray(cards) ? cards : [];
        
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
            onCardClick={(cardId) => {
              if (onCardAction) {
                onCardAction(zoneId, cardId);
              }
            }}
            possibleActions={getCardActions(zoneId)}
          />
        );
      })}
    </div>
  );
}