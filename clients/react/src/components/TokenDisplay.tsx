/**
 * Enhanced Token Display Component
 * 
 * Renders tokens using the new customizable token system while maintaining 
 * backward compatibility with existing token types.
 */

import React, { useEffect, useState } from 'react';
import { TokenManager } from '../tokens/TokenManager';
import { usePlayerToken, useOpponentTokens } from '../hooks/useTokens';
import { usePlayerPreferences } from '../context/PlayerPreferencesContext';

interface TokenDisplayProps {
  // Legacy props for backward compatibility
  tokenType?: string;
  cellSize: number;
  color: string;
  
  // New props for enhanced system
  entityId?: string; // To determine if this is player's or opponent's piece
  playerId?: string; // Which player this token belongs to
  playerIndex?: number; // Index of the player (for opponent colors)
  
  // Optional overrides
  tokenId?: string; // Force specific token
  className?: string;
  
  // Player preferences for token synchronization
  playerPreferences?: Record<string, any>;
}

export const TokenDisplay: React.FC<TokenDisplayProps> = ({
  tokenType,
  cellSize,
  color,
  entityId,
  playerId,
  playerIndex,
  tokenId,
  className = '',
  playerPreferences
}) => {
  const { preferences } = usePlayerPreferences();
  const { currentToken, tokenSvg: playerTokenSvg } = usePlayerToken();
  const { getOpponentToken } = useOpponentTokens();
  const [displaySvg, setDisplaySvg] = useState<string>('');
  const [loading, setLoading] = useState(true);
  
  const tokenSize = cellSize * 0.7;
  
  useEffect(() => {
    const loadTokenDisplay = async () => {
      try {
        const manager = TokenManager.getInstance();
        
        // Determine which token to use
        let finalTokenId: string;
        const finalColor: string = color;
        
        if (tokenId) {
          // Explicit token override
          finalTokenId = tokenId;
        } else if (entityId && preferences) {
          // Check if this is the current player's piece
          const isPlayerPiece = entityId.includes(preferences.username) || 
                                entityId.includes('p1') && playerId === preferences.username;
          
          if (isPlayerPiece && currentToken) {
            // Use player's custom token
            finalTokenId = currentToken.id;
            if (playerTokenSvg) {
              setDisplaySvg(playerTokenSvg);
              setLoading(false);
              return;
            }
          } else if (playerIndex !== undefined) {
            // Use opponent token with proper color
            const opponentToken = await getOpponentToken(playerId || '', playerIndex, playerPreferences);
            setDisplaySvg(opponentToken.svg);
            setLoading(false);
            return;
          }
        }
        
        // Fall back to legacy token system or default
        if (tokenType) {
          finalTokenId = manager.mapLegacyToken(tokenType);
        } else {
          finalTokenId = 'circle'; // Default fallback
        }
        
        // Load the token with color
        const svg = await manager.getColoredSvg(finalTokenId, finalColor);
        setDisplaySvg(svg);
        setLoading(false);
      } catch (error) {
        console.error('Failed to load token display:', error);
        // Create a simple fallback
        const fallbackSvg = `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
          <circle cx="50" cy="50" r="45" fill="${color}"/>
        </svg>`;
        setDisplaySvg(fallbackSvg);
        setLoading(false);
      }
    };
    
    loadTokenDisplay();
  }, [tokenType, cellSize, color, entityId, playerId, playerIndex, tokenId, currentToken, playerTokenSvg, preferences, playerPreferences]);
  
  if (loading) {
    return (
      <div 
        className={`relative ${className}`}
        style={{ 
          width: `${tokenSize}px`, 
          height: `${tokenSize}px`,
          backgroundColor: 'transparent'
        }}
      >
        <div className="animate-pulse bg-gray-600 rounded-full w-full h-full" />
      </div>
    );
  }
  
  return (
    <div 
      className={`relative ${className}`}
      style={{ 
        width: `${tokenSize}px`, 
        height: `${tokenSize}px`,
        backgroundColor: 'transparent'
      }}
      dangerouslySetInnerHTML={{ __html: displaySvg }}
    />
  );
};

// Legacy wrapper for backward compatibility
export function LegacyTokenDisplay({ 
  tokenType, 
  cellSize, 
  color 
}: { 
  tokenType: string; 
  cellSize: number; 
  color: string; 
}) {
  return (
    <TokenDisplay
      tokenType={tokenType}
      cellSize={cellSize}
      color={color}
    />
  );
}

export default TokenDisplay;