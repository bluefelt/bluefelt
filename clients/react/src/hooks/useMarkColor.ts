import { usePlayer } from '../context/PlayerContext';
import { getPlayerColor, PLAYER_COLORS } from '../config/colors';

export const useMarkColor = () => {
  const { player } = usePlayer();
  
  return (cell: any, playerNames?: string[]) => {
    if (!playerNames) return '#888';
    
    // Extract entity ID from cell - could be a string or an object
    let entityId: string;
    if (typeof cell === 'string') {
      entityId = cell;
    } else if (cell && typeof cell === 'object' && cell.entity) {
      entityId = cell.entity;
    } else {
      return '#888';
    }
    
    // Find which player this entity belongs to - matches mark_p1, chip_p1, etc.
    const match = entityId.match(/_p(\d+)$/);
    if (!match) return '#888';
    
    const entityPlayerIndex = parseInt(match[1]) - 1;
    
    if (player) {
      const myPlayerIndex = playerNames.findIndex(name => name === player.username);
      
      if (myPlayerIndex !== -1) {
        // Current player is in the game - use their color preference
        const color = getPlayerColor(entityPlayerIndex, player.color, myPlayerIndex);
        return color.hex;
      }
    }
    
    // Spectator or player not in game - assign colors based on player index
    const colorIndex = entityPlayerIndex % PLAYER_COLORS.length;
    return PLAYER_COLORS[colorIndex].hex;
  };
};