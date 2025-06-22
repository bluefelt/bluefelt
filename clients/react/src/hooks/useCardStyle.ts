/**
 * Card Style Hook - Provides access to card styling functionality
 */

import { useState, useEffect } from 'react';
import { CardStyleRegistry, type CardStyle } from '../cards/CardStyleRegistry';
import { usePlayerPreferences } from '../context/PlayerPreferencesContext';

export function useCardStyle() {
  const { preferences } = usePlayerPreferences();
  const [registry] = useState(() => CardStyleRegistry.getInstance());
  const [currentStyle, setCurrentStyle] = useState<CardStyle | null>(null);

  useEffect(() => {
    if (preferences?.cardStyleId) {
      const style = registry.getStyle(preferences.cardStyleId);
      setCurrentStyle(style || registry.getDefaultStyle());
    } else {
      setCurrentStyle(registry.getDefaultStyle());
    }
  }, [preferences?.cardStyleId, registry]);

  const generateCardBackSVG = (width?: number, height?: number): string => {
    if (!currentStyle) return '';
    return registry.generateCardBackSVG(currentStyle, width, height);
  };

  return {
    currentStyle,
    registry,
    generateCardBackSVG
  };
}

export default useCardStyle;