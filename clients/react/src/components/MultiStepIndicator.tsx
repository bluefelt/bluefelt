import React from 'react';

interface MultiStepIndicatorProps {
  multiStepState: {
    actionId: string;
    actionType: string;
    currentStepId: string;
    currentStepIndex: number;
    totalSteps: number;
    storedData: Record<string, any>;
    canCancel: boolean;
    requiresConfirmation: boolean;
    confirmationPrompt?: string;
  };
}

export const MultiStepIndicator: React.FC<MultiStepIndicatorProps> = ({ multiStepState }) => {
  return (
    <div className="bg-blue-600 text-white p-4 rounded-lg shadow-lg mb-4">
      <div className="text-center">
        <h3 className="text-lg font-bold mb-2">Multi-Step Action Active</h3>
        <p className="text-sm mb-1">Action: {multiStepState.actionId}</p>
        <p className="text-sm mb-1">Current Step: {multiStepState.currentStepId}</p>
        <p className="text-sm">
          Step {multiStepState.currentStepIndex + 1} of {multiStepState.totalSteps}
        </p>
      </div>
    </div>
  );
};