// Central configuration for player colors
export const PLAYER_COLORS = [
  { id: 'pink', name: 'Pink', hex: '#FF1493' },
  { id: 'gold', name: 'Gold', hex: '#FFD700' },
  { id: 'blue', name: 'Blue', hex: '#1E90FF' },
  { id: 'green', name: 'Green', hex: '#32CD32' },
  { id: 'purple', name: 'Purple', hex: '#9370DB' },
  { id: 'orange', name: 'Orange', hex: '#FF8C00' },
  { id: 'cyan', name: 'Cyan', hex: '#00CED1' },
  { id: 'red', name: 'Red', hex: '#DC143C' },
];

export type ColorId = 'pink' | 'gold' | 'blue' | 'green' | 'purple' | 'orange' | 'cyan' | 'red';

export const DEFAULT_COLOR: ColorId = 'pink';

// Player color type
export type PlayerColor = {
  id: ColorId;
  name: string;
  hex: string;
};

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