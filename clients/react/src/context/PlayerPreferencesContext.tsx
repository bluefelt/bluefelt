/**
 * Enhanced Player Preferences Context
 * 
 * Manages player preferences including username, token selection, and color schemes.
 * Maintains backward compatibility with existing PlayerContext.
 */

import { createContext, useContext, useState, useEffect, type ReactNode } from "react";
import type { ColorId } from "../config/colors";
import { DEFAULT_COLOR } from "../config/colors";
import { TokenRegistry } from "../tokens/TokenRegistry";
import { getDefaultColorScheme, type ColorScheme } from "../tokens/ColorSchemes";

export interface UserPreferences {
  username: string;
  // Token preferences
  tokenId: string;
  showOpponentTokens: boolean;
  // Color preferences
  colorSchemeId: string;
  playerColor?: string; // Custom player color within the scheme
  customColorScheme?: ColorScheme;
  // Card style preferences
  cardStyleId: string;
  // Legacy support
  color: ColorId; // Keep for backward compatibility
}

interface PlayerPreferencesContextType {
  preferences: UserPreferences | null;
  isLoggedIn: boolean;
  login: (username: string) => void;
  logout: () => void;
  updatePreferences: (updates: Partial<UserPreferences>) => void;
  updateToken: (tokenId: string) => void;
  updateColorScheme: (schemeId: string) => void;
  updatePlayerColor: (color: string) => void;
  updateShowOpponentTokens: (show: boolean) => void;
  updateCardStyle: (styleId: string) => void;
  // Legacy methods for compatibility
  updateColor: (color: ColorId) => void;
}

const PlayerPreferencesContext = createContext<PlayerPreferencesContextType | undefined>(undefined);

const STORAGE_KEY = 'bluefelt-player-preferences';
const LEGACY_STORAGE_KEY = 'player';

/**
 * Load preferences with migration from old format
 */
function loadPreferences(): UserPreferences | null {
  try {
    // Try to load new format first
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      return JSON.parse(stored);
    }
    
    // Try to migrate from legacy format
    const legacyStored = localStorage.getItem(LEGACY_STORAGE_KEY);
    if (legacyStored) {
      const legacy = JSON.parse(legacyStored);
      const tokenRegistry = TokenRegistry.getInstance();
      
      // Create new format from legacy
      const preferences: UserPreferences = {
        username: legacy.username,
        tokenId: tokenRegistry.getDefaultToken().id,
        showOpponentTokens: true,
        colorSchemeId: getDefaultColorScheme().id,
        cardStyleId: 'classic', // Default card style
        color: legacy.color || DEFAULT_COLOR
      };
      
      // Save in new format
      localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
      
      // Keep legacy format for now (for backward compatibility)
      // Can be removed in future version
      
      return preferences;
    }
  } catch (error) {
    console.error('Failed to load player preferences:', error);
  }
  
  return null;
}

/**
 * Save preferences in both new and legacy formats
 */
function savePreferences(preferences: UserPreferences | null): void {
  if (preferences) {
    // Save new format
    localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences));
    
    // Also save in legacy format for backward compatibility
    localStorage.setItem(LEGACY_STORAGE_KEY, JSON.stringify({
      username: preferences.username,
      color: preferences.color
    }));
  } else {
    localStorage.removeItem(STORAGE_KEY);
    localStorage.removeItem(LEGACY_STORAGE_KEY);
  }
}

export const PlayerPreferencesProvider = ({ children }: { children: ReactNode }) => {
  const [preferences, setPreferences] = useState<UserPreferences | null>(loadPreferences);
  
  // Save preferences whenever they change
  useEffect(() => {
    savePreferences(preferences);
  }, [preferences]);
  
  const login = (username: string) => {
    const tokenRegistry = TokenRegistry.getInstance();
    const newPreferences: UserPreferences = {
      username,
      tokenId: preferences?.tokenId || tokenRegistry.getDefaultToken().id,
      showOpponentTokens: preferences?.showOpponentTokens ?? true,
      colorSchemeId: preferences?.colorSchemeId || getDefaultColorScheme().id,
      cardStyleId: preferences?.cardStyleId || 'classic',
      customColorScheme: preferences?.customColorScheme,
      color: preferences?.color || DEFAULT_COLOR
    };
    setPreferences(newPreferences);
  };
  
  const logout = () => {
    setPreferences(null);
  };
  
  const updatePreferences = (updates: Partial<UserPreferences>) => {
    if (preferences) {
      setPreferences({ ...preferences, ...updates });
    }
  };
  
  const updateToken = (tokenId: string) => {
    updatePreferences({ tokenId });
  };
  
  const updateColorScheme = (schemeId: string) => {
    updatePreferences({ colorSchemeId: schemeId });
  };
  
  const updatePlayerColor = (color: string) => {
    updatePreferences({ playerColor: color });
  };
  
  const updateShowOpponentTokens = (show: boolean) => {
    updatePreferences({ showOpponentTokens: show });
  };
  
  const updateCardStyle = (styleId: string) => {
    updatePreferences({ cardStyleId: styleId });
  };
  
  // Legacy method for backward compatibility
  const updateColor = (color: ColorId) => {
    updatePreferences({ color });
  };
  
  const value: PlayerPreferencesContextType = {
    preferences,
    isLoggedIn: !!preferences,
    login,
    logout,
    updatePreferences,
    updateToken,
    updateColorScheme,
    updatePlayerColor,
    updateShowOpponentTokens,
    updateCardStyle,
    updateColor
  };
  
  return (
    <PlayerPreferencesContext.Provider value={value}>
      {children}
    </PlayerPreferencesContext.Provider>
  );
};

/**
 * Hook to use player preferences
 */
export function usePlayerPreferences() {
  const ctx = useContext(PlayerPreferencesContext);
  if (!ctx) {
    throw new Error("usePlayerPreferences must be used within a PlayerPreferencesProvider");
  }
  return ctx;
}

/**
 * Compatibility layer - provides old PlayerContext interface using new system
 */
export function usePlayer() {
  const { preferences, login: loginPref, logout: logoutPref, updateColor } = usePlayerPreferences();
  
  // Convert to old format
  const player = preferences ? {
    username: preferences.username,
    color: preferences.color
  } : null;
  
  return {
    player,
    login: loginPref,
    logout: logoutPref,
    updateColor
  };
}