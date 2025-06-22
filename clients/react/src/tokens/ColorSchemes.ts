/**
 * Color Scheme System - Manages color palettes with guaranteed contrast
 * 
 * Ensures colors work well together and against black backgrounds.
 */

export interface ColorScheme {
  id: string;
  name: string;
  description: string;
  playerColor: string;
  opponentColors: string[]; // Colors for opponents in order of preference
  backgroundCompatible: boolean; // Works on black background
}

export interface ColorOption {
  id: string;
  name: string;
  hex: string;
  rgb: { r: number; g: number; b: number };
}

// Predefined color schemes with good contrast
export const COLOR_SCHEMES: ColorScheme[] = [
  {
    id: 'warm',
    name: 'Warm Tones',
    description: 'Vibrant warm colors',
    playerColor: '#FFD93D', // Bright yellow
    opponentColors: ['#4ECDC4', '#FF6B6B', '#95E1D3', '#C7CEEA'], // Teal, coral, mint, lavender
    backgroundCompatible: true
  },
  {
    id: 'cool',
    name: 'Cool Tones',
    description: 'Calming cool colors',
    playerColor: '#4ECDC4', // Teal
    opponentColors: ['#FFD93D', '#FF6B6B', '#F38181', '#FFEAA7'], // Yellow, coral, pink, light yellow
    backgroundCompatible: true
  },
  {
    id: 'neon',
    name: 'Neon Brights',
    description: 'High contrast neon colors',
    playerColor: '#00FF88', // Neon green
    opponentColors: ['#FF006E', '#FFBE0B', '#00D9FF', '#C77DFF'], // Neon pink, yellow, cyan, purple
    backgroundCompatible: true
  },
  {
    id: 'pastel',
    name: 'Soft Pastels',
    description: 'Gentle pastel colors',
    playerColor: '#FFB6C1', // Light pink
    opponentColors: ['#87CEEB', '#98FB98', '#DDA0DD', '#F0E68C'], // Sky blue, pale green, plum, khaki
    backgroundCompatible: true
  },
  {
    id: 'classic',
    name: 'Classic Game',
    description: 'Traditional game colors',
    playerColor: '#FF0000', // Red
    opponentColors: ['#0000FF', '#00FF00', '#FFFF00', '#FF00FF'], // Blue, green, yellow, magenta
    backgroundCompatible: true
  },
  {
    id: 'monochrome',
    name: 'Monochrome',
    description: 'Shades of gray with accent',
    playerColor: '#FFFFFF', // White
    opponentColors: ['#CCCCCC', '#999999', '#666666', '#333333'], // Various grays
    backgroundCompatible: true
  }
];

// Individual color options for custom selection
export const COLOR_OPTIONS: ColorOption[] = [
  // Warm colors
  { id: 'red', name: 'Red', hex: '#FF0000', rgb: { r: 255, g: 0, b: 0 } },
  { id: 'coral', name: 'Coral', hex: '#FF6B6B', rgb: { r: 255, g: 107, b: 107 } },
  { id: 'orange', name: 'Orange', hex: '#FFA500', rgb: { r: 255, g: 165, b: 0 } },
  { id: 'amber', name: 'Amber', hex: '#FFA400', rgb: { r: 255, g: 164, b: 0 } },
  { id: 'yellow', name: 'Yellow', hex: '#FFD93D', rgb: { r: 255, g: 217, b: 61 } },
  { id: 'gold', name: 'Gold', hex: '#FFD700', rgb: { r: 255, g: 215, b: 0 } },
  
  // Cool colors
  { id: 'green', name: 'Green', hex: '#00FF00', rgb: { r: 0, g: 255, b: 0 } },
  { id: 'emerald', name: 'Emerald', hex: '#26D07C', rgb: { r: 38, g: 208, b: 124 } },
  { id: 'teal', name: 'Teal', hex: '#4ECDC4', rgb: { r: 78, g: 205, b: 196 } },
  { id: 'cyan', name: 'Cyan', hex: '#00D9FF', rgb: { r: 0, g: 217, b: 255 } },
  { id: 'blue', name: 'Blue', hex: '#0000FF', rgb: { r: 0, g: 0, b: 255 } },
  { id: 'indigo', name: 'Indigo', hex: '#4B0082', rgb: { r: 75, g: 0, b: 130 } },
  
  // Purple/Pink
  { id: 'purple', name: 'Purple', hex: '#800080', rgb: { r: 128, g: 0, b: 128 } },
  { id: 'violet', name: 'Violet', hex: '#B19CD9', rgb: { r: 177, g: 156, b: 217 } },
  { id: 'magenta', name: 'Magenta', hex: '#FF00FF', rgb: { r: 255, g: 0, b: 255 } },
  { id: 'rose', name: 'Rose', hex: '#FF6FCF', rgb: { r: 255, g: 111, b: 207 } },
  { id: 'pink', name: 'Pink', hex: '#FFB6C1', rgb: { r: 255, g: 182, b: 193 } },
  
  // Neutral
  { id: 'white', name: 'White', hex: '#FFFFFF', rgb: { r: 255, g: 255, b: 255 } },
  { id: 'silver', name: 'Silver', hex: '#C0C0C0', rgb: { r: 192, g: 192, b: 192 } },
  { id: 'gray', name: 'Gray', hex: '#808080', rgb: { r: 128, g: 128, b: 128 } }
];

/**
 * Calculate relative luminance for WCAG contrast
 */
export function getLuminance(color: string): number {
  // Convert hex to RGB if needed
  let r: number, g: number, b: number;
  
  if (color.startsWith('#')) {
    const hex = color.slice(1);
    r = parseInt(hex.substr(0, 2), 16) / 255;
    g = parseInt(hex.substr(2, 2), 16) / 255;
    b = parseInt(hex.substr(4, 2), 16) / 255;
  } else {
    // Assume it's already in a parsed format
    return 0;
  }
  
  // Apply gamma correction
  const gammaCorrect = (channel: number): number => {
    return channel <= 0.03928
      ? channel / 12.92
      : Math.pow((channel + 0.055) / 1.055, 2.4);
  };
  
  const rg = gammaCorrect(r);
  const gg = gammaCorrect(g);
  const bg = gammaCorrect(b);
  
  // Calculate relative luminance
  return 0.2126 * rg + 0.7152 * gg + 0.0722 * bg;
}

/**
 * Calculate contrast ratio between two colors
 */
export function getContrastRatio(color1: string, color2: string): number {
  const l1 = getLuminance(color1);
  const l2 = getLuminance(color2);
  
  const lighter = Math.max(l1, l2);
  const darker = Math.min(l1, l2);
  
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Check if two colors have sufficient contrast
 */
export function hasGoodContrast(
  color1: string,
  color2: string,
  minRatio: number = 3.0 // WCAG AA for large text
): boolean {
  return getContrastRatio(color1, color2) >= minRatio;
}

/**
 * Get colors that contrast well with a given color
 */
export function getContrastingColors(
  baseColor: string,
  candidateColors: string[],
  minRatio: number = 3.0
): string[] {
  return candidateColors.filter(color => 
    hasGoodContrast(baseColor, color, minRatio)
  );
}

/**
 * Ensure a color works on black background
 */
export function worksOnBlackBackground(color: string): boolean {
  return hasGoodContrast(color, '#000000', 3.0);
}

/**
 * Get a color scheme by ID
 */
export function getColorScheme(id: string): ColorScheme | undefined {
  return COLOR_SCHEMES.find(scheme => scheme.id === id);
}

/**
 * Get default color scheme
 */
export function getDefaultColorScheme(): ColorScheme {
  return COLOR_SCHEMES[0]; // Warm tones by default
}

/**
 * Create a custom color scheme ensuring good contrast
 */
export function createCustomColorScheme(
  playerColor: string,
  availableColors: string[] = COLOR_OPTIONS.map(c => c.hex)
): ColorScheme {
  // Filter colors that work on black background
  const validColors = availableColors.filter(worksOnBlackBackground);
  
  // Get colors that contrast with player color
  const opponentColors = getContrastingColors(playerColor, validColors)
    .filter(color => color !== playerColor)
    .slice(0, 4); // Take top 4
  
  // If we don't have enough contrasting colors, add some safe defaults
  if (opponentColors.length < 4) {
    const safeDefaults = ['#FFFFFF', '#FFD93D', '#4ECDC4', '#FF6B6B'];
    safeDefaults.forEach(color => {
      if (opponentColors.length < 4 && 
          color !== playerColor && 
          hasGoodContrast(playerColor, color)) {
        opponentColors.push(color);
      }
    });
  }
  
  return {
    id: 'custom',
    name: 'Custom Scheme',
    description: 'User-defined color scheme',
    playerColor,
    opponentColors,
    backgroundCompatible: worksOnBlackBackground(playerColor)
  };
}