import { useState } from 'react';
import { useCardStyle } from '../hooks/useCardStyle';
import { usePlayerCardStyles } from '../hooks/usePlayerCardStyles';

interface CardProps {
  suit?: string;
  rank?: string;
  isFaceUp: boolean;
  isSelectable?: boolean;
  isSelected?: boolean;
  onClick?: (e: React.MouseEvent) => void;
  size?: 'small' | 'medium' | 'large';
  // Props for card style customization
  playerId?: string; // Which player this card belongs to
  playerPreferences?: Record<string, any>; // All player preferences for style lookup
}

const SUIT_SYMBOLS: Record<string, string> = {
  hearts: '♥',
  diamonds: '♦',
  clubs: '♣',
  spades: '♠'
};

const SUIT_COLORS: Record<string, string> = {
  hearts: '#FF0000',
  diamonds: '#FF0000',
  clubs: '#000000',
  spades: '#000000'
};

export default function Card({ 
  suit = 'hearts', 
  rank = 'A', 
  isFaceUp = true, 
  isSelectable = false,
  isSelected = false,
  onClick,
  size = 'medium',
  playerId,
  playerPreferences
}: CardProps) {
  const [isHovered, setIsHovered] = useState(false);
  const { currentStyle, generateCardBackSVG } = useCardStyle();
  const { getPlayerCardStyle, getPlayerCardBackSVG } = usePlayerCardStyles(playerPreferences);

  const sizeStyles = {
    small: { width: 60, height: 84, fontSize: 22 },
    medium: { width: 70, height: 105, fontSize: 26 },
    large: { width: 90, height: 135, fontSize: 34 }
  };

  const currentSize = sizeStyles[size];
  const symbol = SUIT_SYMBOLS[suit] || '?';
  const color = SUIT_COLORS[suit] || '#000000';

  // Determine which card style to use based on player
  const effectiveStyle = playerId ? getPlayerCardStyle(playerId) : currentStyle;
  const effectiveGenerateCardBack = playerId ? 
    (w?: number, h?: number) => getPlayerCardBackSVG(playerId, w, h) : 
    generateCardBackSVG;

  // Get card style colors
  const frontColors = effectiveStyle?.front.colors || {
    background: '#FFFFFF',
    border: '#000000',
    text: '#000000'
  };

  const cardStyle: React.CSSProperties = {
    width: currentSize.width,
    height: currentSize.height,
    borderRadius: effectiveStyle?.front.cornerStyle === 'rounded' ? 12 : 8,
    border: `2px solid ${isSelected ? '#FFD700' : isSelectable ? '#6B8BFF' : (isFaceUp ? frontColors.border : '#333')}`,
    backgroundColor: isFaceUp ? frontColors.background : 'transparent',
    cursor: isSelectable ? 'pointer' : 'default',
    transition: 'all 0.2s ease',
    transform: isHovered && isSelectable ? 'translateY(-5px)' : 'none',
    boxShadow: isSelectable ? '0 0 10px rgba(107, 139, 255, 0.5)' : isHovered && isSelectable ? '0 5px 15px rgba(0,0,0,0.3)' : '0 2px 5px rgba(0,0,0,0.2)',
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    position: 'relative',
    userSelect: 'none',
    overflow: 'hidden'
  };

  const rankStyle: React.CSSProperties = {
    fontSize: currentSize.fontSize,
    fontWeight: 'bold',
    color: isFaceUp ? color : frontColors.text,
    lineHeight: 1
  };

  const suitStyle: React.CSSProperties = {
    fontSize: currentSize.fontSize * 0.8,
    color: isFaceUp ? color : frontColors.text,
    marginTop: 4
  };

  const cornerStyle: React.CSSProperties = {
    position: 'absolute',
    fontSize: currentSize.fontSize * 0.5,
    fontWeight: 'bold',
    color: isFaceUp ? color : frontColors.text,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    lineHeight: 1
  };

  return (
    <div 
      style={cardStyle}
      onClick={isSelectable ? onClick : undefined}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {isFaceUp ? (
        <>
          {/* Top left corner */}
          <div style={{ ...cornerStyle, top: 4, left: 4 }}>
            <span>{rank}</span>
            <span style={{ fontSize: currentSize.fontSize * 0.4 }}>{symbol}</span>
          </div>
          
          {/* Bottom right corner (rotated) */}
          <div style={{ ...cornerStyle, bottom: 4, right: 4, transform: 'rotate(180deg)' }}>
            <span>{rank}</span>
            <span style={{ fontSize: currentSize.fontSize * 0.4 }}>{symbol}</span>
          </div>
          
          {/* Center */}
          <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
            <span style={rankStyle}>{rank}</span>
            <span style={suitStyle}>{symbol}</span>
          </div>
        </>
      ) : (
        // Card back design using custom style
        <div 
          className="w-full h-full"
          dangerouslySetInnerHTML={{ __html: effectiveGenerateCardBack(currentSize.width, currentSize.height) }}
        />
      )}
    </div>
  );
}