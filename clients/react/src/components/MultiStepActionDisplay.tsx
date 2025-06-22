import React from 'react';
import { Button } from './ui/Button';
import { Card } from './ui/Card';

interface MultiStepState {
  actionId: string;
  actionType: string;
  currentStepId: string;
  currentStepIndex: number;
  totalSteps: number;
  storedData: Record<string, any>;
  canCancel: boolean;
  requiresConfirmation: boolean;
  confirmationPrompt?: string;
  direction?: string;
}

interface MultiStepActionDisplayProps {
  multiStepState: MultiStepState | null;
  onCancel: () => void;
  onConfirm: () => void;
}

export const MultiStepActionDisplay: React.FC<MultiStepActionDisplayProps> = ({
  multiStepState,
  onCancel,
  onConfirm,
}) => {
  if (!multiStepState) {
    return null;
  }

  const { 
    currentStepIndex, 
    totalSteps, 
    canCancel, 
    requiresConfirmation,
    confirmationPrompt,
    actionType 
  } = multiStepState;

  // Get action-specific icon
  const getActionIcon = (actionType: string) => {
    switch (actionType) {
      case 'movePiece':
      case 'makeMove':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16l-4-4m0 0l4-4m-4 4h18" />
          </svg>
        );
      case 'askForCard':
      case 'playCard':
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        );
      default:
        return (
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        );
    }
  };

  // If this is a confirmation step
  if (requiresConfirmation && confirmationPrompt) {
    return (
      <Card 
        className="fixed top-20 right-4 p-4 shadow-lg max-w-md z-50 bg-white border border-amber-200"
        role="dialog"
        aria-labelledby="confirmation-title"
        aria-describedby="confirmation-message"
      >
        <div className="space-y-4">
          <div className="flex items-center space-x-3">
            <div 
              className="flex-shrink-0 w-8 h-8 bg-amber-100 rounded-full flex items-center justify-center"
              aria-hidden="true"
            >
              <svg className="w-5 h-5 text-amber-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <h3 id="confirmation-title" className="text-lg font-semibold text-gray-900">Confirm Action</h3>
          </div>
          
          {/* Progress indicator for confirmation */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm text-gray-600">
              <span>Final Step</span>
              <span className="font-medium text-green-600">Ready to complete</span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-2">
              <div className="bg-gradient-to-r from-green-500 to-green-600 h-2 rounded-full w-full" />
            </div>
          </div>
          
          <div className="p-3 bg-amber-50 rounded-lg border border-amber-200">
            <p id="confirmation-message" className="text-sm text-amber-800 font-medium">{confirmationPrompt}</p>
          </div>
          
          <div className="flex gap-2 justify-end" role="group" aria-label="Action confirmation buttons">
            {canCancel && (
              <Button
                variant="outline"
                size="sm"
                onClick={onCancel}
                className="border-gray-300 text-gray-700 hover:bg-gray-50"
                aria-label="Cancel action"
              >
                Cancel
              </Button>
            )}
            <Button
              size="sm"
              onClick={onConfirm}
              className="bg-green-600 hover:bg-green-700 text-white"
              aria-label="Confirm and complete action"
            >
              Confirm
            </Button>
          </div>
        </div>
      </Card>
    );
  }

  // Regular multi-step progress display
  return (
    <Card 
      className="fixed top-20 right-4 p-4 shadow-lg max-w-md z-50 bg-white border border-gray-200"
      role="status"
      aria-labelledby="progress-title"
      aria-describedby="progress-description"
      aria-live="polite"
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 id="progress-title" className="text-lg font-semibold text-gray-900">Action in Progress</h3>
          {canCancel && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onCancel}
              className="text-red-600 hover:text-red-700 hover:bg-red-50"
              aria-label="Cancel current action"
            >
              Cancel
            </Button>
          )}
        </div>
        
        {/* Enhanced progress indicator with step visualization */}
        <div className="space-y-3">
          {/* Step indicators */}
          <div 
            className="flex items-center justify-center" 
            role="progressbar" 
            aria-valuenow={currentStepIndex + 1} 
            aria-valuemin={1} 
            aria-valuemax={totalSteps}
            aria-label={`Step ${currentStepIndex + 1} of ${totalSteps}`}
          >
            {Array.from({ length: totalSteps }, (_, index) => (
              <React.Fragment key={index}>
                <div className="flex flex-col items-center">
                  <div
                    className={`w-8 h-8 rounded-full border-2 flex items-center justify-center text-sm font-medium transition-all duration-300 ${
                      index < currentStepIndex
                        ? 'bg-blue-600 border-blue-600 text-white'
                        : index === currentStepIndex
                        ? 'bg-blue-100 border-blue-600 text-blue-600 ring-2 ring-blue-200'
                        : 'bg-gray-100 border-gray-300 text-gray-400'
                    }`}
                    aria-label={
                      index < currentStepIndex 
                        ? `Step ${index + 1} completed`
                        : index === currentStepIndex
                        ? `Step ${index + 1} current`
                        : `Step ${index + 1} pending`
                    }
                  >
                    {index < currentStepIndex ? (
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                        <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                      </svg>
                    ) : (
                      index + 1
                    )}
                  </div>
                </div>
                {index < totalSteps - 1 && (
                  <div
                    className={`h-0.5 w-12 mx-2 transition-all duration-300 ${
                      index < currentStepIndex ? 'bg-blue-600' : 'bg-gray-300'
                    }`}
                    aria-hidden="true"
                  />
                )}
              </React.Fragment>
            ))}
          </div>
          
          {/* Progress bar with percentage */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm text-gray-600">
              <span>Step {currentStepIndex + 1} of {totalSteps}</span>
              <span className="font-medium text-blue-600">
                {Math.round(((currentStepIndex + 1) / totalSteps) * 100)}%
              </span>
            </div>
            <div className="w-full bg-gray-200 rounded-full h-3 overflow-hidden" role="progressbar" aria-hidden="true">
              <div 
                className="bg-gradient-to-r from-blue-500 to-blue-600 h-3 rounded-full transition-all duration-500 ease-out"
                style={{ width: `${((currentStepIndex + 1) / totalSteps) * 100}%` }}
              />
            </div>
          </div>
        </div>

        {/* Step instructions with action-specific icon */}
        <div 
          className="flex items-start space-x-3 p-3 bg-blue-50 rounded-lg border border-blue-200"
          id="progress-description"
        >
          <div className="flex-shrink-0 w-5 h-5 mt-0.5 text-blue-600" aria-hidden="true">
            {getActionIcon(actionType)}
          </div>
          <p className="text-sm text-blue-800 font-medium">
            {multiStepState.direction || getStepInstruction(multiStepState)}
          </p>
        </div>
      </div>
    </Card>
  );
};

// Helper function to generate user-friendly step instructions
function getStepInstruction(state: MultiStepState): string {
  const { currentStepId, actionType, storedData } = state;
  
  // Generate contextual instructions based on action type and step
  switch (actionType) {
    case 'movePiece':
    case 'movePieceToEmptyLocation':
      if (currentStepId === 'selectPiece') {
        return 'Select a piece to move';
      } else if (currentStepId === 'selectDestination') {
        const piece = storedData.selectedPiece;
        return piece ? `Select where to move ${piece}` : 'Select destination';
      }
      break;
    case 'askForCard':
      if (currentStepId === 'selectRank') {
        return 'Choose which rank to ask for';
      } else if (currentStepId === 'selectPlayer') {
        const rank = storedData.selectedRank;
        return rank ? `Choose who to ask for ${rank}s` : 'Choose which player to ask';
      }
      break;
    case 'makeMove':
      if (currentStepId === 'selectFrom') {
        return 'Select the piece you want to move';
      } else if (currentStepId === 'selectTo') {
        return 'Select where to move the piece';
      }
      break;
    case 'playCard':
      if (currentStepId === 'selectCard') {
        return 'Select a card to play';
      } else if (currentStepId === 'selectTarget') {
        return 'Select the target for your card';
      }
      break;
  }
  
  // Default instruction with better formatting
  const formattedStepId = currentStepId
    .replace(/([A-Z])/g, ' $1')
    .replace(/^./, str => str.toUpperCase())
    .trim();
  return formattedStepId;
}