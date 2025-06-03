import { useCallback } from 'react';
import { getPlayerEntity } from '../utils/entityUtils';

interface GameActionHookProps {
  isYourTurn: boolean;
  lobbyState: any;
  sendMessage: (message: string) => boolean;
}

export function useGameActions({ isYourTurn, lobbyState, sendMessage }: GameActionHookProps) {
  // Handle board cell clicks (and column clicks)
  const handleCellClick = useCallback((row: number, col: number) => {
    console.log('[CLIENT] handleCellClick called:', { row, col, isYourTurn, you: lobbyState.you });
    
    if (!isYourTurn || !lobbyState.you) {
      console.log('[CLIENT] Early return - not your turn or no player ID');
      return;
    }
    
    const playerActions = lobbyState.ui?.actionMap?.[lobbyState.you] || {};
    console.log('[CLIENT] Player actions for', lobbyState.you, ':', playerActions);
    console.log('[CLIENT] Current phase:', lobbyState.game?.phases?.game?.current);
    console.log('[CLIENT] Full action map:', lobbyState.ui?.actionMap);
    
    // Check if this is a column action (row === -1)
    if (row === -1) {
      const columnLocation = `/zones/board/columns/${col}`;
      const columnAction = playerActions[columnLocation];
      
      if (columnAction) {
        // Get the correct entity for this player
        const playerNum = parseInt(lobbyState.you.substring(1)); // "p1" -> 1
        const playerEntity = getPlayerEntity(lobbyState.ui?.entities, playerNum);
        const entityId = playerEntity?.id || `disc_${lobbyState.you}`;
        
        // This is a column-based action (like placeWithGravity)
        const message = JSON.stringify({
          action: columnAction.action,
          args: {
            zone: "/zones/board",
            column: col,
            entity: entityId
          }
        });
        sendMessage(message);
      }
      return;
    }
    
    // Standard cell action
    const location = `/zones/board/cells/${row}/${col}`;
    const action = playerActions[location];
    console.log('[CLIENT] Checking location:', location, 'action:', action);
    
    if (action) {
      console.log('[CLIENT] Found action:', action.action, 'for location:', location);
      let args;
      
      // Handle different action types with appropriate arguments
      switch (action.action) {
        case 'selectPiece':
        case 'selectEntity':
          // Selection actions need location and player
          args = {
            location: location,
            player: lobbyState.you
          };
          break;
          
        case 'moveSelectedPiece':
        case 'moveSelected':
          // Move actions need target and player
          args = {
            target: location,
            player: lobbyState.you
          };
          break;
          
        case 'clearSelection':
          // Clear selection just needs player
          args = {
            player: lobbyState.you
          };
          break;
          
        default:
          // Default to place-style actions (location + entity)
          const playerNum = parseInt(lobbyState.you.substring(1)); // "p1" -> 1
          const playerEntity = getPlayerEntity(lobbyState.ui?.entities, playerNum);
          const entityId = playerEntity?.id || `mark_${lobbyState.you}`;
          args = {
            location: location,
            entity: entityId
          };
          break;
      }
      
      const message = JSON.stringify({
        action: action.action,
        args: args
      });
      console.log('[CLIENT] Sending message:', message);
      const result = sendMessage(message);
      console.log('[CLIENT] SendMessage result:', result);
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