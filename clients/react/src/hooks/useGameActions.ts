import { useCallback } from 'react';

interface GameActionHookProps {
  isYourTurn: boolean;
  lobbyState: any;
  sendMessage: (message: string) => boolean;
}

export function useGameActions({ isYourTurn, lobbyState, sendMessage }: GameActionHookProps) {
  // Handle board cell clicks (and column clicks)
  const handleCellClick = useCallback((row: number, col: number) => {
    if (!isYourTurn || !lobbyState.you) {
      return;
    }
    
    const playerActions = lobbyState.ui?.actionMap?.[lobbyState.you] || {};
    
    // Check if this is a column action (row === -1)
    if (row === -1) {
      const columnLocation = `/zones/board/columns/${col}`;
      const columnAction = playerActions[columnLocation];
      
      if (columnAction) {
        // This is a column-based action (like placeWithGravity)
        const message = JSON.stringify({
          action: columnAction.action,
          args: {
            zone: "/zones/board",
            column: col,
            entity: `disc_${lobbyState.you}` // For Connect 4
          }
        });
        sendMessage(message);
      }
      return;
    }
    
    // Standard cell action
    const location = `/zones/board/cells/${row}/${col}`;
    const action = playerActions[location];
    
    if (action) {
      // For place actions, we need to send location and entity
      const entity = `mark_${lobbyState.you}`; // e.g., "mark_p1"
      const message = JSON.stringify({
        action: action.action,
        args: {
          location: location,
          entity: entity
        }
      });
      sendMessage(message);
    }
  }, [isYourTurn, lobbyState, sendMessage]);

  // Handle card actions
  const handleCardAction = useCallback((zoneId: string, cardIndex: number) => {
    if (!isYourTurn || !lobbyState.you) return;
    
    const playerActions = lobbyState.ui?.actionMap?.[lobbyState.you] || {};
    
    // Check for zone-level action (cardIndex === -1)
    if (cardIndex === -1) {
      const zoneLocation = `/zones/${zoneId}`;
      const zoneAction = playerActions[zoneLocation];
      if (zoneAction) {
        sendMessage(JSON.stringify({
          action: zoneAction.action,
          args: zoneAction.args || { location: zoneLocation }
        }));
      }
      return;
    }
    
    // Card-specific action
    const location = `/zones/${zoneId}/${cardIndex}`;
    const action = playerActions[location];
    
    if (action) {
      const args = action.args || { location };
      sendMessage(JSON.stringify({
        action: action.action,
        args: args
      }));
    }
  }, [isYourTurn, lobbyState, sendMessage]);

  // Handle general zone actions
  const handleZoneAction = useCallback((location: string, actionName: string) => {
    if (!isYourTurn || !lobbyState.you) return;
    
    const playerActions = lobbyState.ui?.actionMap?.[lobbyState.you] || {};
    const action = playerActions[location];
    
    if (action && action.action === actionName) {
      sendMessage(JSON.stringify({
        action: action.action,
        args: { location }
      }));
    }
  }, [isYourTurn, lobbyState, sendMessage]);

  return {
    handleCellClick,
    handleCardAction,
    handleZoneAction
  };
}