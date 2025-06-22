/**
 * Centralized utilities for action map processing
 * Reduces code duplication and provides consistent action handling
 */

import type { LobbyState } from '../ws/useLobbyWebSocket';

export interface Action {
  action: string;
  direction?: string;
  args?: Record<string, any>;
  multiStepId?: string;
}

/**
 * Get the current player's action map with safe null handling
 */
export function getPlayerActionMap(lobbyState: LobbyState): Record<string, Action> {
  return lobbyState.game?.ui?.actionMap?.[lobbyState.you || ''] || {};
}

/**
 * Get standard action locations - simplified to single canonical paths
 * Server should provide standardized paths to eliminate multiple format support
 */
export function getCellActionLocation(zoneId: string, row: number, col: number): string {
  return `/zones/${zoneId}/cells/${row}/${col}`;
}

export function getCardActionLocation(zoneId: string, cardIndex: number): string {
  return `/zones/${zoneId}/items/${cardIndex}`;
}

export function getChoiceActionLocation(zoneId: string, choice: string): string {
  return `/zones/${zoneId}/${choice}`;
}

/**
 * Get zone-level action location
 */
export function getZoneActionLocation(zoneId: string): string {
  return `/zones/${zoneId}`;
}

/**
 * Check if a location has a valid action
 */
export function hasActionAtLocation(
  actionMap: Record<string, Action>, 
  location: string
): boolean {
  return location in actionMap;
}

/**
 * Get action at specific location (checks both path formats for compatibility)
 */
export function getActionAtLocation(
  actionMap: Record<string, Action>, 
  location: string
): Action | null {
  // First try the exact location
  if (actionMap[location]) {
    return actionMap[location];
  }
  
  // If it's a cell location, try the alternative format
  const cellMatch = location.match(/^\/zones\/([^/]+)\/cells\/(\d+)\/(\d+)$/);
  if (cellMatch) {
    const [, zoneId, row, col] = cellMatch;
    const altLocation = `/zones/${zoneId}/${row}/${col}`;
    return actionMap[altLocation] || null;
  }
  
  // If it's the short format, try the cells format
  const shortMatch = location.match(/^\/zones\/([^/]+)\/(\d+)\/(\d+)$/);
  if (shortMatch) {
    const [, zoneId, row, col] = shortMatch;
    const altLocation = `/zones/${zoneId}/cells/${row}/${col}`;
    return actionMap[altLocation] || null;
  }
  
  return null;
}

/**
 * Validate if an action can be executed considering multi-step state
 */
export function canExecuteAction(
  action: Action,
  multiStepState: any,
  isInMultiStep: boolean
): boolean {
  if (!isInMultiStep) return true;
  if (!multiStepState) return true;
  return action.multiStepId === multiStepState.actionId;
}

/**
 * Get the first available action from action map (for turn prompts)
 */
export function getFirstAction(actionMap: Record<string, Action>): Action | null {
  const actions = Object.values(actionMap);
  return actions.length > 0 ? actions[0] : null;
}

/**
 * Check if an action is a multi-step selection
 */
export function isMultiStepSelect(action: Action): boolean {
  return action.action === 'multiStepSelect';
}

/**
 * Execute an action by sending it via WebSocket
 */
export function executeAction(
  action: Action,
  sendMessage: (message: string) => void
): void {
  // Debug log what we're about to send
  console.log('[executeAction] Sending action:', {
    action: action.action,
    args: action.args,
    fullAction: action
  });
  
  // Handle multi-step selections differently
  if (isMultiStepSelect(action)) {
    const message = {
      action: 'multiStepSelect',
      args: action.args || {}
    };
    console.log('[executeAction] MultiStep message:', message);
    sendMessage(JSON.stringify(message));
  } else {
    // Regular action
    const message = {
      action: action.action,
      args: action.args || {}
    };
    console.log('[executeAction] Regular message:', message);
    sendMessage(JSON.stringify(message));
  }
}