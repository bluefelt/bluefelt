/**
 * Player Card Styles Hook - Provides card styling for players including opponents
 */

import { useState, useEffect } from 'react';
import { CardStyleRegistry, type CardStyle } from '../cards/CardStyleRegistry';
import { usePlayerPreferences } from '../context/PlayerPreferencesContext';

interface PlayerCardStyles {
  [playerId: string]: {
    style: CardStyle;
    generateCardBackSVG: (width?: number, height?: number) => string;
  };
}

export function usePlayerCardStyles(playerPreferences?: Record<string, any>) {
  const { preferences } = usePlayerPreferences();
  const [registry] = useState(() => CardStyleRegistry.getInstance());
  const [playerCardStyles, setPlayerCardStyles] = useState<PlayerCardStyles>({});

  useEffect(() => {
    const styles: PlayerCardStyles = {};

    // Add current player's card style
    if (preferences?.cardStyleId) {
      const currentStyle = registry.getStyle(preferences.cardStyleId) || registry.getDefaultStyle();
      styles['current'] = {
        style: currentStyle,
        generateCardBackSVG: (width, height) => registry.generateCardBackSVG(currentStyle, width, height)
      };
    }

    // Add opponent card styles from synchronized preferences
    if (playerPreferences) {
      Object.entries(playerPreferences).forEach(([playerId, prefs]) => {
        if (prefs && typeof prefs === 'object' && 'cardStyleId' in prefs) {
          const cardStyleId = prefs.cardStyleId;
          if (cardStyleId && typeof cardStyleId === 'string') {
            const opponentStyle = registry.getStyle(cardStyleId) || registry.getDefaultStyle();
            styles[playerId] = {
              style: opponentStyle,
              generateCardBackSVG: (width, height) => registry.generateCardBackSVG(opponentStyle, width, height)
            };
          }
        }
      });
    }

    setPlayerCardStyles(styles);
  }, [preferences?.cardStyleId, playerPreferences, registry]);

  const getPlayerCardStyle = (playerId?: string): CardStyle => {
    if (!playerId || playerId === 'current') {
      return playerCardStyles['current']?.style || registry.getDefaultStyle();
    }
    
    return playerCardStyles[playerId]?.style || registry.getDefaultStyle();
  };

  const getPlayerCardBackSVG = (playerId?: string, width?: number, height?: number): string => {
    if (!playerId || playerId === 'current') {
      return playerCardStyles['current']?.generateCardBackSVG(width, height) || 
             registry.generateCardBackSVG(registry.getDefaultStyle(), width, height);
    }
    
    return playerCardStyles[playerId]?.generateCardBackSVG(width, height) || 
           registry.generateCardBackSVG(registry.getDefaultStyle(), width, height);
  };

  return {
    playerCardStyles,
    getPlayerCardStyle,
    getPlayerCardBackSVG,
    registry
  };
}

export default usePlayerCardStyles;