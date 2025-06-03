import Card from './Card';
import { getPlayerColor } from '../config/colors';

interface CardEntity {
  id: string;
  props?: {
    suit?: string;
    rank?: string;
    value?: number;
  };
}

interface CardZoneProps {
  zoneId: string;
  zoneName: string;
  cards: CardEntity[];
  layout: 'stack' | 'fan' | 'spread';
  visibility: 'all' | 'owner' | 'none';
  isOwner: boolean;
  showCount?: boolean;
  showTop?: boolean;
  onCardClick?: (cardId: string, cardIndex: number) => void;
  onZoneClick?: () => void;
  possibleActions?: Array<{
    cardIndex: number;
    action: string;
  }>;
  hasZoneAction?: boolean;
  playerNames?: string[];
  you?: string;
  playerColor?: string;
}

export default function CardZone({
  zoneName,
  cards,
  layout,
  visibility,
  isOwner,
  showCount = false,
  showTop = false,
  onCardClick,
  onZoneClick,
  possibleActions = [],
  hasZoneAction = false,
  playerNames = [],
  you,
  playerColor
}: CardZoneProps) {

  // Determine if cards should be face up
  const getCardVisibility = (index: number): boolean => {
    if (visibility === 'all') return true;
    if (visibility === 'owner' && isOwner) return true;
    if (visibility === 'none') return false;
    
    // For stack layout with showTop, only show the last card
    if (layout === 'stack' && showTop && index === cards.length - 1) {
      return true;
    }
    
    return false;
  };

  // Calculate card positions based on layout
  const getCardStyle = (index: number, total: number): React.CSSProperties => {
    const baseStyle: React.CSSProperties = {
      position: 'absolute',
      transition: 'all 0.3s ease'
    };

    switch (layout) {
      case 'stack':
        // For stack layout, only show the top card
        if (index === total - 1) {
          return {
            ...baseStyle,
            left: 0,
            top: 0,
            zIndex: index
          };
        } else {
          // Hide all other cards
          return {
            ...baseStyle,
            display: 'none'
          };
        }
      
      case 'fan': {
        // Fan cards side by side with slight rotation
        const cardWidth = 60; // Width of small cards
        const gap = 8; // Gap between cards
        const totalWidth = total * cardWidth + (total - 1) * gap;
        const startX = -totalWidth / 2;
        
        // Slight rotation for visual effect
        const maxRotation = 5; // degrees
        const rotationStep = (2 * maxRotation) / Math.max(total - 1, 1);
        const rotation = -maxRotation + (index * rotationStep);
        
        return {
          ...baseStyle,
          left: `calc(50% + ${startX + index * (cardWidth + gap)}px)`,
          bottom: 10,
          transform: `translateX(0) rotate(${rotation}deg)`,
          transformOrigin: 'bottom center',
          zIndex: index
        };
      }
      
      case 'spread': {
        // Spread cards horizontally without overlap
        const cardWidth = 60; // Width of small cards
        const gap = 8; // Gap between cards
        return {
          ...baseStyle,
          left: index * (cardWidth + gap),
          top: 0,
          zIndex: index
        };
      }
      
      default:
        return baseStyle;
    }
  };

  const containerStyle: React.CSSProperties = {
    position: 'relative',
    minHeight: layout === 'fan' ? 140 : 110,
    width: '100%',
    margin: '10px',
    padding: '10px',
    backgroundColor: 'rgba(0, 0, 0, 0.2)',
    borderRadius: 8,
    border: '2px solid #444',
    overflow: (layout === 'spread' || layout === 'fan') ? 'auto' : 'hidden'
  };

  // Parse zone name to replace player placeholders with actual names
  const parseZoneName = (name: string): React.ReactNode => {
    // Check if the name contains player placeholders like {p1}, {p2}, etc.
    const playerMatch = name.match(/\{(p\d+)\}/);
    
    if (playerMatch) {
      const playerId = playerMatch[1]; // e.g., "p1"
      const playerIndex = parseInt(playerId.substring(1)) - 1; // Convert p1 to 0, p2 to 1, etc.
      const playerName = playerNames[playerIndex] || `Player ${playerIndex + 1}`;
      
      // Get the player's color
      const myIndex = you ? parseInt(you.substring(1)) - 1 : 0;
      const playerColorObj = getPlayerColor(playerIndex, playerColor as any || 'coral', myIndex);
      
      // Replace the placeholder with styled player name
      const parts = name.split(playerMatch[0]);
      return (
        <>
          {parts[0]}
          <span style={{ color: playerColorObj.hex, fontWeight: 'bold' }}>
            {playerName}
          </span>
          {parts[1]}
        </>
      );
    }
    
    return name;
  };

  const labelStyle: React.CSSProperties = {
    fontSize: 14,
    color: '#D8B260',
    fontFamily: 'Roboto Condensed, sans-serif',
    marginBottom: 8,
    fontWeight: 'bold'
  };

  const countStyle: React.CSSProperties = {
    fontSize: 16,
    color: '#D8B260',
    fontFamily: 'Roboto Condensed, sans-serif',
    fontWeight: 'bold',
    marginLeft: 10
  };

  // For stack layout with hidden cards, show placeholder
  if (layout === 'stack' && visibility === 'none' && !showTop) {
    const clickableStyle: React.CSSProperties = hasZoneAction && possibleActions.length === 0 ? {
      cursor: 'pointer',
      border: '2px solid #6B8BFF',
      boxShadow: '0 0 10px rgba(107, 139, 255, 0.5)'
    } : {};
    
    return (
      <div 
        style={{...containerStyle, ...clickableStyle}}
        onClick={hasZoneAction ? onZoneClick : undefined}
      >
        <div style={labelStyle}>{parseZoneName(zoneName)}</div>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
          {cards.length > 0 && (
            <Card 
              isFaceUp={false}
              size="medium"
              isSelectable={hasZoneAction && possibleActions.length === 0}
            />
          )}
          {showCount && cards.length > 0 && <div style={countStyle}>{cards.length}</div>}
        </div>
      </div>
    );
  }

  // For other layouts including stack with showTop (discard pile)
  const clickableStyle: React.CSSProperties = hasZoneAction && layout === 'stack' && possibleActions.length === 0 ? {
    cursor: 'pointer',
    border: '2px solid #6B8BFF',
    boxShadow: '0 0 10px rgba(107, 139, 255, 0.5)'
  } : {};
  
  return (
    <div 
      style={{...containerStyle, ...clickableStyle}}
      onClick={hasZoneAction && layout === 'stack' ? onZoneClick : undefined}
    >
      <div style={labelStyle}>{parseZoneName(zoneName)}</div>
      {showCount && layout !== 'stack' && <div style={countStyle}>{cards.length} cards</div>}
      
      <div style={{ 
        position: 'relative', 
        width: layout === 'stack' ? 'auto' : '100%', 
        height: layout === 'fan' ? '120px' : '110px',
        margin: '0 auto',
        display: layout === 'stack' ? 'flex' : 'block',
        alignItems: layout === 'stack' ? 'center' : 'normal',
        justifyContent: layout === 'stack' ? 'center' : 'normal'
      }}>
        {cards.map((card, index) => {
          const isFaceUp = getCardVisibility(index);
          // For stack layout, make the top card selectable if there's a zone action
          const isTopCard = index === cards.length - 1;
          const isSelectable = possibleActions.some(action => action.cardIndex === index) ||
            (hasZoneAction && layout === 'stack' && isTopCard);
          
          return (
            <div
              key={card.id}
              style={getCardStyle(index, cards.length)}
            >
              <Card
                suit={card.props?.suit}
                rank={card.props?.rank}
                isFaceUp={isFaceUp}
                isSelectable={isSelectable}
                onClick={(e) => {
                  e.stopPropagation(); // Prevent event bubbling to container
                  if (possibleActions.some(action => action.cardIndex === index)) {
                    onCardClick?.(card.id, index);
                  } else if (hasZoneAction && layout === 'stack' && isTopCard) {
                    onZoneClick?.();
                  }
                }}
                size="medium"
              />
            </div>
          );
        })}
        {showCount && layout === 'stack' && cards.length > 0 && <div style={countStyle}>{cards.length}</div>}
      </div>
    </div>
  );
}