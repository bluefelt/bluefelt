export type ColorId = 'coral' | 'emerald' | 'gold' | 'azure' | 'rose' | 'lime' | 'violet' | 'amber';

// Player color type
export type PlayerColor = {
  id: ColorId;
  name: string;
  hex: string;
};

// Central configuration for player colors
// Ordered to maximize visual distinction between adjacent colors
export const PLAYER_COLORS: PlayerColor[] = [
  { id: 'coral', name: 'Coral', hex: '#FF6B6B' },     // Warm red-orange
  { id: 'emerald', name: 'Emerald', hex: '#26D07C' }, // Vibrant green
  { id: 'gold', name: 'Gold', hex: '#FFD93D' },       // Bright yellow
  { id: 'rose', name: 'Rose', hex: '#FF6FCF' },       // Hot pink
  { id: 'amber', name: 'Amber', hex: '#FFA400' },     // Orange
  { id: 'violet', name: 'Violet', hex: '#B19CD9' },   // Light purple
];

export const DEFAULT_COLOR: ColorId = 'coral';

// Get color by ID
export function getColorById(colorId: ColorId | undefined): PlayerColor {
  return PLAYER_COLORS.find(c => c.id === colorId) || PLAYER_COLORS[0];
}

// Get opponent color (next color in the list, wrapping around)
export function getOpponentColor(myColorId: ColorId): PlayerColor {
  const myColorIndex = PLAYER_COLORS.findIndex(c => c.id === myColorId);
  const opponentIndex = (myColorIndex + 1) % PLAYER_COLORS.length;
  return PLAYER_COLORS[opponentIndex];
}

// Get color for a player based on their position and my color
export function getPlayerColor(playerIndex: number, myColorId: ColorId, myPlayerIndex: number): PlayerColor {
  if (playerIndex === myPlayerIndex) {
    return getColorById(myColorId);
  }
  
  // For opponents, cycle through colors starting from the next color after mine
  const myColorIndex = PLAYER_COLORS.findIndex(c => c.id === myColorId);
  const offset = playerIndex > myPlayerIndex ? playerIndex - myPlayerIndex : playerIndex - myPlayerIndex + 2;
  const colorIndex = (myColorIndex + offset) % PLAYER_COLORS.length;
  return PLAYER_COLORS[colorIndex];
}