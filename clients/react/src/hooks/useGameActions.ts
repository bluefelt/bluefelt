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
    console.log('[CLIENT] Keys in player actions:', Object.keys(playerActions));
    
    // Check if this is a column action (row === -1)
    if (row === -1) {
      const columnLocation = `/zones/board/columns/${col}`;
      const columnAction = playerActions[columnLocation];
      
      if (columnAction) {
        // Get the correct entity for this player
        const playerNum = parseInt(lobbyState.you.substring(1)); // "p1" -> 1
        const playerEntity = getPlayerEntity(lobbyState.ui?.entities, playerNum);
        const entityId = playerEntity?.id || `disc_${lobbyState.you}`;
        
        // Handle different column action types
        let args: any = {};
        
        switch (columnAction.action) {
          case 'dropDisc':
            // Connect Four expects zone, targetColumn, entity
            args = {
              zone: '/zones/board',
              targetColumn: columnAction.targetColumn || col,
              entity: entityId
            };
            break;
            
          default:
            // Default behavior - use targetColumn and player
            if ('targetColumn' in columnAction) {
              args.targetColumn = columnAction.targetColumn;
            } else if ('column' in columnAction) {
              args.targetColumn = columnAction.column;
            } else {
              args.targetColumn = col;
            }
            args.player = lobbyState.you;
            break;
        }
        
        const message = JSON.stringify({
          action: columnAction.action,
          args
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
          
        case 'placeToken':
          // Three Men's Morris uses target + entity with piece entities
          const playerNum = parseInt(lobbyState.you.substring(1)); // "p1" -> 1
          const playerEntity = getPlayerEntity(lobbyState.ui?.entities, playerNum);
          const entityId = playerEntity?.id || `piece_${lobbyState.you}`;
          args = {
            target: location,
            entity: entityId
          };
          break;
          
        default:
          // Default to place-style actions (location + entity)
          const playerNum2 = parseInt(lobbyState.you.substring(1)); // "p1" -> 1
          const playerEntity2 = getPlayerEntity(lobbyState.ui?.entities, playerNum2);
          const entityId2 = playerEntity2?.id || `mark_${lobbyState.you}`;
          args = {
            location: location,
            entity: entityId2
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

  // Handle choice selections (for games like Go Fish)
  const handleChoiceSelect = useCallback((zoneId: string, choice: string) => {
    if (!isYourTurn || !lobbyState.you) return;
    
    console.log('[CLIENT] handleChoiceSelect called:', { zoneId, choice, you: lobbyState.you });
    
    const playerActions = lobbyState.ui?.actionMap?.[lobbyState.you] || {};
    
    // First, try to find the action in the action map
    // The location could be in various formats depending on the choice zone structure
    const possibleLocations = [
      `/zones/${zoneId}/${choice}`,                    // Direct choice
      `/zones/${zoneId}/ranks/${choice}`,              // Rank selection
      `/zones/${zoneId}/players/${choice}`,            // Player selection
      `/zones/choice_${lobbyState.you}/ranks/${choice}`,    // Player-specific rank zone
      `/zones/choice_${lobbyState.you}/players/${choice}`   // Player-specific player zone
    ];
    
    let action = null;
    let location = null;
    
    // Try each possible location format
    for (const loc of possibleLocations) {
      if (playerActions[loc]) {
        action = playerActions[loc];
        location = loc;
        break;
      }
    }
    
    if (action) {
      console.log('[CLIENT] Found action in map:', action, 'for location:', location);
      
      // Use the args from the action map if provided, otherwise construct them
      const args = action.args || {
        // For rank selection
        ...(action.rank ? { rank: action.rank, player: lobbyState.you } : {}),
        // For player selection  
        ...(action.targetPlayer ? { targetPlayer: action.targetPlayer, player: lobbyState.you } : {}),
        // Generic fallback
        ...(!action.rank && !action.targetPlayer ? { 
          selection: choice,
          zone: zoneId,
          player: lobbyState.you 
        } : {})
      };
      
      const message = JSON.stringify({ 
        action: action.action, 
        args 
      });
      console.log('[CLIENT] Sending choice message:', message);
      sendMessage(message);
    } else {
      console.warn('[CLIENT] No action found in map for choice:', { zoneId, choice, possibleLocations });
      
      // Fallback: infer action and args from zone ID and choice
      let fallbackAction = null;
      let fallbackArgs = null;
      
      if (zoneId === 'ranks') {
        fallbackAction = 'selectRank';
        fallbackArgs = { rank: choice, player: lobbyState.you };
      } else if (zoneId === 'players') {
        fallbackAction = 'selectPlayer';
        fallbackArgs = { targetPlayer: choice, player: lobbyState.you };
      }
      
      if (fallbackAction && fallbackArgs) {
        const message = JSON.stringify({ 
          action: fallbackAction, 
          args: fallbackArgs 
        });
        console.log('[CLIENT] Using fallback choice action:', message);
        sendMessage(message);
      }
    }
  }, [isYourTurn, lobbyState, sendMessage]);

  return {
    handleCellClick,
    handleCardAction,
    handleZoneAction,
    handleChoiceSelect
  };
}