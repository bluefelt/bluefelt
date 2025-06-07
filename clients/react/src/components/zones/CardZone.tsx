import React from 'react';

interface CardZoneProps {
  id: string;
  cards: Array<{ entity: string; [key: string]: any }>;
  entityDefinitions?: Record<string, { 
    id: string; 
    name?: string; 
    props?: { 
      rank?: string; 
      suit?: string; 
      faceUp?: boolean;
      color?: string;
      borderStyle?: string;
      [key: string]: any;
    };
  }>;
  onCardClick?: (zoneId: string, cardIndex: number) => void;
  onZoneClick?: () => void;
  metadata?: {
    shape?: 'list' | 'stack' | 'single' | 'spread';
    placeholder?: string;
    type?: string;
    label?: string;
  };
  visibility?: 'all' | 'back' | 'count';
  you?: string;
  possibleActions?: Record<string, { action: string; direction: string }>;
  hasZoneAction?: boolean;
}

const getSuitSymbol = (suit: string): string => {
  const symbols: Record<string, string> = {
    hearts: '♥',
    diamonds: '♦',
    clubs: '♣',
    spades: '♠'
  };
  return symbols[suit] || suit;
};

const getSuitColor = (suit: string): string => {
  return suit === 'hearts' || suit === 'diamonds' ? 'text-red-600' : 'text-black';
};

export const CardZone: React.FC<CardZoneProps> = ({
  id,
  cards,
  entityDefinitions = {},
  onCardClick,
  onZoneClick,
  metadata = {},
  visibility = 'all',
  you,
  possibleActions = {},
  hasZoneAction = false
}) => {
  const { shape = 'list', placeholder, type, label } = metadata;

  // Determine if this is a player-specific zone and if we should hide cards
  const isPlayerZone = id.includes('_p') && you;
  const isOwnZone = isPlayerZone && id.includes(`_${you}`);
  const shouldShowFaces = visibility === 'all' || isOwnZone;
  const shouldShowCount = visibility === 'count' && !shouldShowFaces;
  const shouldShowBacks = visibility === 'back' || (isPlayerZone && !isOwnZone && visibility !== 'count');

  // Handle empty zone
  if (cards.length === 0) {
    return (
      <div 
        className={`border-2 border-dashed border-gray-300 rounded-lg p-4 min-h-16 flex items-center justify-center ${
          hasZoneAction && onZoneClick ? 'cursor-pointer hover:border-gray-400' : ''
        }`}
        onClick={hasZoneAction && onZoneClick ? onZoneClick : undefined}
        role={hasZoneAction && onZoneClick ? 'button' : undefined}
      >
        <span className="text-gray-500 text-sm">
          {placeholder || `Empty ${id}`}
        </span>
      </div>
    );
  }

  // Handle count-only display
  if (shouldShowCount) {
    return (
      <div className="border border-gray-300 rounded-lg p-3 bg-gray-50">
        <span className="text-sm text-gray-600">
          {cards.length} card{cards.length !== 1 ? 's' : ''}
        </span>
      </div>
    );
  }

  // Handle stack display
  if (shape === 'stack') {
    return (
      <div className="relative">
        {label && <div className="text-sm font-medium mb-2">{label}</div>}
        <div className="relative w-16 h-24 bg-blue-800 rounded border-2 border-gray-400 flex items-center justify-center text-white font-bold">
          {cards.length}
          {/* Stack effect with multiple card shadows */}
          <div className="absolute -top-1 -right-1 w-full h-full bg-blue-700 rounded border border-gray-400 -z-10"></div>
          <div className="absolute -top-2 -right-2 w-full h-full bg-blue-600 rounded border border-gray-400 -z-20"></div>
        </div>
      </div>
    );
  }

  // Render individual cards
  const renderCard = (card: { entity: string; [key: string]: any }, index: number) => {
    const entityDef = entityDefinitions[card.entity];
    const props = entityDef?.props || {};
    const action = possibleActions[index.toString()];
    
    const cardName = entityDef?.name || card.entity;
    const rank = props.rank;
    const suit = props.suit;
    const faceUp = props.faceUp !== false; // Default to face up
    
    // Determine if this card should show face or back
    const showFace = shouldShowFaces && faceUp;
    const showBack = shouldShowBacks || !faceUp;

    // Custom styling
    const customColor = props.color;
    const borderStyle = props.borderStyle;
    
    const cardStyle: React.CSSProperties = {
      backgroundColor: customColor,
      borderWidth: borderStyle === 'thick' ? '3px' : '1px'
    };

    const handleCardClick = () => {
      if (onCardClick) {
        onCardClick(id, index);
      }
    };

    if (showBack) {
      return (
        <button
          key={index}
          className="w-12 h-16 bg-blue-800 rounded border border-gray-400 flex items-center justify-center text-white text-lg hover:bg-blue-700"
          onClick={handleCardClick}
          title={action?.direction}
          style={cardStyle}
        >
          🂠
        </button>
      );
    }

    if (showFace && rank && suit) {
      const suitSymbol = getSuitSymbol(suit);
      const suitColor = getSuitColor(suit);
      
      return (
        <button
          key={index}
          className={`w-12 h-16 bg-white rounded border border-gray-400 flex flex-col items-center justify-center text-xs hover:bg-gray-50 ${suitColor}`}
          onClick={handleCardClick}
          title={action?.direction || cardName}
          style={cardStyle}
        >
          <span className="font-bold">{rank}</span>
          <span className="text-lg">{suitSymbol}</span>
        </button>
      );
    }

    // Fallback to entity name
    return (
      <button
        key={index}
        className="w-12 h-16 bg-white rounded border border-gray-400 flex items-center justify-center text-xs p-1 hover:bg-gray-50"
        onClick={handleCardClick}
        title={action?.direction}
        style={cardStyle}
      >
        <span className="text-center leading-tight">{cardName}</span>
      </button>
    );
  };

  const cardElements = cards.map(renderCard);

  // Render based on shape
  return (
    <div>
      {label && <div className="text-sm font-medium mb-2">{label}</div>}
      
      {shape === 'single' ? (
        <div className="flex justify-center">
          {cardElements[0]}
        </div>
      ) : shape === 'spread' ? (
        <div className="flex space-x-1 flex-wrap">
          {cardElements}
        </div>
      ) : (
        // Default 'list' layout
        <div className="flex space-x-1 overflow-x-auto max-w-full">
          {cardElements}
        </div>
      )}
    </div>
  );
};

export default CardZone;