import React from 'react';

interface ActionIndicatorProps {
  hasAction: boolean;
  isMultiStep?: boolean;
  multiStepState?: 'available' | 'current_step' | 'next_step' | 'selected' | 'confirmed';
  stepNumber?: number;
  className?: string;
}

export const ActionIndicator: React.FC<ActionIndicatorProps> = ({
  hasAction,
  isMultiStep = false,
  multiStepState = 'available',
  stepNumber,
  className
}) => {
  if (!hasAction) return null;

  const baseClasses = "absolute inset-0 pointer-events-none";
  
  // Enhanced visual states for multi-step actions
  const getMultiStepStyles = () => {
    switch (multiStepState) {
      case 'current_step':
        return {
          ring: "absolute inset-0 rounded ring-4 ring-yellow-400 ring-opacity-90 animate-pulse",
          corner: "absolute top-0 right-0 w-3 h-3 rounded-full bg-yellow-400 animate-bounce",
          glow: "absolute inset-0 rounded bg-yellow-400 bg-opacity-20 animate-pulse"
        };
      case 'next_step':
        return {
          ring: "absolute inset-0 rounded ring-2 ring-blue-400 ring-opacity-70 animate-pulse",
          corner: "absolute top-0 right-0 w-2 h-2 rounded-full bg-blue-400 animate-pulse",
          glow: "absolute inset-0 rounded bg-blue-400 bg-opacity-10"
        };
      case 'selected':
        return {
          ring: "absolute inset-0 rounded ring-3 ring-green-500 ring-opacity-80",
          corner: "absolute top-0 right-0 w-3 h-3 rounded-full bg-green-500",
          glow: "absolute inset-0 rounded bg-green-500 bg-opacity-20",
          checkmark: true
        };
      case 'confirmed':
        return {
          ring: "absolute inset-0 rounded ring-2 ring-green-600 ring-opacity-100",
          corner: "absolute top-0 right-0 w-3 h-3 rounded-full bg-green-600",
          glow: "absolute inset-0 rounded bg-green-600 bg-opacity-30",
          checkmark: true
        };
      default: // 'available'
        return {
          ring: "absolute inset-0 rounded ring-2 ring-purple-500 ring-opacity-75 animate-pulse",
          corner: "absolute top-0 right-0 w-2 h-2 rounded-full bg-purple-500 animate-ping",
          glow: "absolute inset-0 rounded bg-purple-500 bg-opacity-10"
        };
    }
  };

  const standardStyles = {
    ring: "absolute inset-0 rounded ring-2 ring-blue-500 ring-opacity-50 animate-pulse",
    corner: "absolute top-0 right-0 w-2 h-2 rounded-full bg-blue-500 animate-ping",
    glow: "absolute inset-0 rounded bg-blue-500 bg-opacity-10"
  };

  const styles = isMultiStep ? getMultiStepStyles() : standardStyles;

  return (
    <div className={`${baseClasses} ${className || ''}`}>
      {/* Background glow effect */}
      <div className={styles.glow} />
      
      {/* Pulsing border effect for available actions */}
      <div className={styles.ring} />
      
      {/* Corner indicator with step number or checkmark */}
      <div className={styles.corner}>
        {styles.checkmark ? (
          <svg 
            className="w-full h-full text-white" 
            fill="currentColor" 
            viewBox="0 0 20 20"
          >
            <path 
              fillRule="evenodd" 
              d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" 
              clipRule="evenodd" 
            />
          </svg>
        ) : stepNumber ? (
          <span className="text-xs font-bold text-white flex items-center justify-center w-full h-full">
            {stepNumber}
          </span>
        ) : null}
      </div>
      
      {/* Additional visual effects for current step */}
      {multiStepState === 'current_step' && (
        <>
          <div className="absolute inset-0 rounded bg-gradient-to-br from-yellow-400 to-yellow-600 opacity-10" />
          <div className="absolute top-1 left-1 w-1 h-1 rounded-full bg-yellow-400 animate-ping" />
          <div className="absolute bottom-1 right-1 w-1 h-1 rounded-full bg-yellow-400 animate-ping" style={{ animationDelay: '0.5s' }} />
        </>
      )}
    </div>
  );
};

// Hook to determine if an element has an action
export function useHasAction(
  location: string,
  playerActions: Record<string, any>,
  isInMultiStep: boolean,
  multiStepState?: any
): { 
  hasAction: boolean; 
  isMultiStepAction: boolean;
  multiStepIndicatorState: 'available' | 'current_step' | 'next_step' | 'selected' | 'confirmed';
  stepNumber?: number;
} {
  // Check regular action map first
  let action = playerActions[location];
  
  // If no action in regular map and we're in multi-step mode, check stepActionMap
  if (!action && isInMultiStep && multiStepState?.stepActionMap) {
    action = multiStepState.stepActionMap[location];
  }
  
  if (!action) {
    return { 
      hasAction: false, 
      isMultiStepAction: false,
      multiStepIndicatorState: 'available'
    };
  }
  
  // Check if this is a multi-step action
  const isMultiStepAction = Boolean(
    isInMultiStep && 
    multiStepState && 
    action.multiStepId === multiStepState.actionId
  );
  
  // In multi-step mode, only show indicators for current multi-step actions
  if (isInMultiStep && !isMultiStepAction) {
    return { 
      hasAction: false, 
      isMultiStepAction: false,
      multiStepIndicatorState: 'available'
    };
  }
  
  // Determine multi-step visual state
  let multiStepIndicatorState: 'available' | 'current_step' | 'next_step' | 'selected' | 'confirmed' = 'available';
  let stepNumber: number | undefined;
  
  if (isMultiStepAction && multiStepState) {
    // Check if this location was already selected in a previous step
    const isAlreadySelected = multiStepState.storedData && 
      Object.values(multiStepState.storedData).some((value: any) => {
        if (typeof value === 'string') {
          return value === location || value.includes(`${location.split('/').pop()}`);
        }
        if (value && typeof value === 'object' && value.location) {
          return value.location === location;
        }
        return false;
      });
    
    if (isAlreadySelected) {
      multiStepIndicatorState = multiStepState.requiresConfirmation ? 'confirmed' : 'selected';
    } else if (multiStepState.stepActionMap && multiStepState.stepActionMap[location]) {
      // This is available for the current step
      multiStepIndicatorState = 'current_step';
      
      // Try to determine step number from the multi-step state
      if (multiStepState.currentStepIndex !== undefined) {
        stepNumber = multiStepState.currentStepIndex + 1;
      }
    } else {
      // This might be available for a future step
      multiStepIndicatorState = 'next_step';
    }
  }
  
  return { 
    hasAction: true, 
    isMultiStepAction,
    multiStepIndicatorState,
    stepNumber
  };
}