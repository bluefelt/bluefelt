import { useCallback } from 'react';
import {
  getPlayerActionMap,
  getCellActionLocation,
  getCardActionLocation,
  getChoiceActionLocation,
  getZoneActionLocation,
  getActionAtLocation,
  canExecuteAction as canExecuteActionUtil,
  executeAction
} from '../utils/actionMapUtils';

interface GameActionHookProps {
  isYourTurn: boolean;
  lobbyState: any;
  sendMessage: (message: string) => boolean;
}

export function useGameActions({ isYourTurn, lobbyState, sendMessage }: GameActionHookProps) {
  // Get current multi-step state for validation
  const multiStepState = lobbyState.ui?.multiStepState?.[lobbyState.you || ''];
  const isInMultiStep = Boolean(multiStepState);
  
  // Helper to check if action should be allowed
  const canExecuteAction = useCallback((action: any) => {
    // During multi-step, use the step's action map if available
    if (isInMultiStep && multiStepState?.stepActionMap) {
      // Action must be in the current step's action map
      const stepActions = Object.values(multiStepState.stepActionMap);
      return stepActions.some((stepAction: any) => 
        JSON.stringify(stepAction.args) === JSON.stringify(action.args)
      );
    }
    return canExecuteActionUtil(action, multiStepState, isInMultiStep);
  }, [multiStepState, isInMultiStep]);
  // Handle board cell clicks (and column clicks) using server authority
  const handleCellClick = useCallback((row: number, col: number) => {
    // During multi-step, allow actions even if not your turn
    if (!isInMultiStep && (!isYourTurn || !lobbyState.you)) return;
    
    // Use step action map during multi-step, otherwise use regular action map
    const playerActions = isInMultiStep && multiStepState?.stepActionMap 
      ? multiStepState.stepActionMap 
      : getPlayerActionMap(lobbyState);
    
    // Debug logging
    console.log('[useGameActions] handleCellClick called:', { row, col, isYourTurn });
    console.log('[useGameActions] playerActions:', playerActions);
    console.log('[useGameActions] available action locations:', Object.keys(playerActions));
    
    // Check if this is a column action (row === -1)
    if (row === -1) {
      const action = playerActions[`/zones/board/columns/${col}`];
      if (action && canExecuteAction(action)) {
        executeAction(action, sendMessage);
      }
      return;
    }
    
    // Standard cell action
    const location = getCellActionLocation('board', row, col);
    const action = getActionAtLocation(playerActions, location);
    
    console.log('[useGameActions] Looking for action at location:', location);
    console.log('[useGameActions] Found action:', action);
    
    if (action && canExecuteAction(action)) {
      console.log('[useGameActions] Executing action:', action);
      executeAction(action, sendMessage);
    } else {
      console.log('[useGameActions] No valid action found for location:', location);
    }
  }, [isYourTurn, isInMultiStep, multiStepState, lobbyState, sendMessage, canExecuteAction]);

  // Handle card actions using server authority
  const handleCardAction = useCallback((zoneId: string, cardIndex: number) => {
    // During multi-step, allow actions even if not your turn
    if (!isInMultiStep && (!isYourTurn || !lobbyState.you)) return;
    
    // Use step action map during multi-step, otherwise use regular action map
    const playerActions = isInMultiStep && multiStepState?.stepActionMap 
      ? multiStepState.stepActionMap 
      : getPlayerActionMap(lobbyState);
    
    // Check for zone-level action (cardIndex === -1)
    if (cardIndex === -1) {
      const action = playerActions[getZoneActionLocation(zoneId)];
      if (action && canExecuteAction(action)) {
        executeAction(action, sendMessage);
      }
      return;
    }
    
    // Card-specific action
    const location = getCardActionLocation(zoneId, cardIndex);
    const action = getActionAtLocation(playerActions, location);
    
    if (action && canExecuteAction(action)) {
      executeAction(action, sendMessage);
    }
  }, [isYourTurn, isInMultiStep, multiStepState, lobbyState, sendMessage, canExecuteAction]);

  // Handle general zone actions using server authority
  const handleZoneAction = useCallback((location: string, actionName: string) => {
    // During multi-step, allow actions even if not your turn
    if (!isInMultiStep && (!isYourTurn || !lobbyState.you)) return;
    
    // Use step action map during multi-step, otherwise use regular action map
    const playerActions = isInMultiStep && multiStepState?.stepActionMap 
      ? multiStepState.stepActionMap 
      : getPlayerActionMap(lobbyState);
    const action = playerActions[location];
    
    if (action && action.action === actionName && canExecuteAction(action)) {
      executeAction(action, sendMessage);
    }
  }, [isYourTurn, isInMultiStep, multiStepState, lobbyState, sendMessage, canExecuteAction]);

  // Handle choice selections using server authority
  const handleChoiceSelect = useCallback((zoneId: string, choice: string) => {
    // During multi-step, allow actions even if not your turn
    if (!isInMultiStep && (!isYourTurn || !lobbyState.you)) return;
    
    // Use step action map during multi-step, otherwise use regular action map
    const playerActions = isInMultiStep && multiStepState?.stepActionMap 
      ? multiStepState.stepActionMap 
      : getPlayerActionMap(lobbyState);
    
    // For multi-step selections, use the appropriate path format
    let location;
    if (isInMultiStep) {
      if (multiStepState?.actionType === 'bf.selectChoice') {
        // For dynamic choices like ranks, the action map uses /ranks/{choice}
        // The choice ID is already the correct value from the stepActionMap
        location = `/ranks/${choice}`;
      } else if (multiStepState?.actionType === 'bf.selectPlayer') {
        // For player selection, use /players/{player}
        location = `/players/${choice}`;
      } else {
        location = getChoiceActionLocation(zoneId, choice);
      }
    } else {
      location = getChoiceActionLocation(zoneId, choice);
    }
    
    const action = getActionAtLocation(playerActions, location);
    
    if (action && canExecuteAction(action)) {
      executeAction(action, sendMessage);
    } else {
      console.warn(`No action found for choice "${choice}" in zone "${zoneId}" at location "${location}". Server should provide complete action map.`);
    }
  }, [isYourTurn, isInMultiStep, multiStepState, lobbyState, sendMessage, canExecuteAction]);

  return {
    handleCellClick,
    handleCardAction,
    handleZoneAction,
    handleChoiceSelect
  };
}