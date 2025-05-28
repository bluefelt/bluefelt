import Card from './Card';

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
  possibleActions?: Array<{
    cardId: string;
    verb: string;
  }>;
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
  possibleActions = []
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
  
  console.log('CardZone render:', {
    zoneName,
    cardsCount: cards.length,
    cards: cards.map(c => c.id),
    visibility,
    isOwner,
    layout
  });

  // Calculate card positions based on layout
  const getCardStyle = (index: number, total: number): React.CSSProperties => {
    const baseStyle: React.CSSProperties = {
      position: 'absolute',
      transition: 'all 0.3s ease'
    };

    switch (layout) {
      case 'stack':
        // Stack cards with slight offset
        return {
          ...baseStyle,
          left: index * 2,
          top: index * 2,
          zIndex: index
        };
      
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

  const labelStyle: React.CSSProperties = {
    fontSize: 14,
    color: '#D8B260',
    fontFamily: 'Roboto Condensed, sans-serif',
    marginBottom: 8,
    fontWeight: 'bold'
  };

  const countStyle: React.CSSProperties = {
    position: 'absolute',
    top: 10,
    right: 10,
    fontSize: 12,
    color: '#999',
    fontFamily: 'Roboto Condensed, sans-serif'
  };

  // For stack layout with hidden cards, show placeholder
  if (layout === 'stack' && visibility === 'none' && !showTop) {
    return (
      <div style={containerStyle}>
        <div style={labelStyle}>{zoneName}</div>
        {showCount && <div style={countStyle}>{cards.length} cards</div>}
        {cards.length > 0 && (
          <Card 
            isFaceUp={false}
            size="medium"
          />
        )}
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      <div style={labelStyle}>{zoneName}</div>
      {showCount && <div style={countStyle}>{cards.length} cards</div>}
      
      <div style={{ 
        position: 'relative', 
        width: '100%', 
        height: layout === 'fan' ? '120px' : '110px',
        margin: '0 auto'
      }}>
        {cards.map((card, index) => {
          const isFaceUp = getCardVisibility(index);
          const isSelectable = possibleActions.some(action => action.cardId === card.id);
          
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
                onClick={() => isSelectable && onCardClick?.(card.id, index)}
                size="medium"
              />
            </div>
          );
        })}
      </div>
    </div>
  );
}