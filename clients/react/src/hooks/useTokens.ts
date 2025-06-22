/**
 * React hooks for token system
 */

import { useState, useEffect, useCallback } from 'react';
import { TokenManager } from '../tokens/TokenManager';
import { TokenRegistry, type TokenDefinition } from '../tokens/TokenRegistry';
import { usePlayerPreferences } from '../context/PlayerPreferencesContext';
import { getColorScheme, type ColorScheme } from '../tokens/ColorSchemes';

interface UseTokenResult {
  currentToken: TokenDefinition | null;
  tokenSvg: string | null;
  loading: boolean;
  error: string | null;
}

/**
 * Hook to get current player's token with SVG
 */
export function usePlayerToken(): UseTokenResult {
  const { preferences } = usePlayerPreferences();
  const [result, setResult] = useState<UseTokenResult>({
    currentToken: null,
    tokenSvg: null,
    loading: true,
    error: null
  });
  
  useEffect(() => {
    if (!preferences) {
      setResult({
        currentToken: null,
        tokenSvg: null,
        loading: false,
        error: null
      });
      return;
    }
    
    const loadToken = async () => {
      try {
        const registry = TokenRegistry.getInstance();
        const manager = TokenManager.getInstance();
        
        const token = registry.getToken(preferences.tokenId);
        if (!token) {
          throw new Error(`Token ${preferences.tokenId} not found`);
        }
        
        // Get player's color from color scheme
        const scheme = getColorScheme(preferences.colorSchemeId) || 
                      preferences.customColorScheme;
        const color = scheme?.playerColor || '#FFFFFF';
        
        const svg = await manager.getColoredSvg(token.id, color);
        
        setResult({
          currentToken: token,
          tokenSvg: svg,
          loading: false,
          error: null
        });
      } catch (error) {
        console.error('Failed to load player token:', error);
        setResult({
          currentToken: null,
          tokenSvg: null,
          loading: false,
          error: error instanceof Error ? error.message : 'Failed to load token'
        });
      }
    };
    
    loadToken();
  }, [preferences?.tokenId, preferences?.colorSchemeId]);
  
  return result;
}

interface UseOpponentTokenResult {
  getOpponentToken: (playerId: string, playerIndex: number, playerPreferences?: Record<string, any>) => Promise<{
    token: TokenDefinition;
    svg: string;
    color: string;
  }>;
}

/**
 * Hook to get opponent tokens with proper colors
 */
export function useOpponentTokens(): UseOpponentTokenResult {
  const { preferences } = usePlayerPreferences();
  
  const getOpponentToken = useCallback(async (
    playerId: string,
    playerIndex: number,
    playerPreferences?: Record<string, any>
  ) => {
    const registry = TokenRegistry.getInstance();
    const manager = TokenManager.getInstance();
    
    // If not showing opponent tokens, use default
    if (!preferences?.showOpponentTokens) {
      const defaultToken = registry.getDefaultToken();
      const scheme = getColorScheme(preferences?.colorSchemeId || 'warm');
      const color = scheme?.opponentColors[playerIndex % scheme.opponentColors.length] || '#FFFFFF';
      
      const svg = await manager.getColoredSvg(defaultToken.id, color);
      
      return {
        token: defaultToken,
        svg,
        color
      };
    }
    
    // Try to get opponent's actual preferences from synchronized data
    const opponentPrefs = playerPreferences?.[playerId];
    if (opponentPrefs) {
      // Use opponent's actual token preference
      const tokenId = opponentPrefs.tokenId || 'circle';
      const token = registry.getToken(tokenId) || registry.getDefaultToken();
      
      // Use opponent's custom color if available, otherwise use their scheme color
      let color = '';
      if (opponentPrefs.playerColor) {
        color = opponentPrefs.playerColor;
      } else if (opponentPrefs.colorSchemeId) {
        const opponentScheme = getColorScheme(opponentPrefs.colorSchemeId);
        color = opponentScheme?.playerColor || '#FFFFFF';
      } else {
        // Fallback to our scheme's opponent color
        const scheme = getColorScheme(preferences?.colorSchemeId || 'warm');
        color = scheme?.opponentColors[playerIndex % scheme.opponentColors.length] || '#FFFFFF';
      }
      
      const svg = await manager.getColoredSvg(token.id, color);
      
      return {
        token,
        svg,
        color
      };
    }
    
    // Fallback: use different basic shapes for opponents if no preferences available
    const opponentTokens = ['square', 'triangle', 'diamond', 'hexagon'];
    const tokenId = opponentTokens[playerIndex % opponentTokens.length];
    const token = registry.getToken(tokenId) || registry.getDefaultToken();
    
    const scheme = getColorScheme(preferences?.colorSchemeId || 'warm');
    const color = scheme?.opponentColors[playerIndex % scheme.opponentColors.length] || '#FFFFFF';
    
    const svg = await manager.getColoredSvg(token.id, color);
    
    return {
      token,
      svg,
      color
    };
  }, [preferences?.showOpponentTokens, preferences?.colorSchemeId]);
  
  return { getOpponentToken };
}

/**
 * Hook to get all available tokens
 */
export function useAvailableTokens() {
  const [tokens, setTokens] = useState<TokenDefinition[]>([]);
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const registry = TokenRegistry.getInstance();
    setTokens(registry.getAllTokens());
    setLoading(false);
  }, []);
  
  return { tokens, loading };
}

/**
 * Hook to preload tokens for better performance
 */
export function useTokenPreloader(tokenIds: string[]) {
  const [loaded, setLoaded] = useState(false);
  
  useEffect(() => {
    if (tokenIds.length === 0) {
      setLoaded(true);
      return;
    }
    
    const manager = TokenManager.getInstance();
    manager.preloadTokens(tokenIds)
      .then(() => setLoaded(true))
      .catch(error => {
        console.error('Failed to preload tokens:', error);
        setLoaded(true); // Continue anyway
      });
  }, [tokenIds.join(',')]);
  
  return loaded;
}