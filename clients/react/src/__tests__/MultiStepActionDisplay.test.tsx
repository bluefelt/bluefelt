import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import { MultiStepActionDisplay } from '../components/MultiStepActionDisplay';

// Mock data for tests
const mockMultiStepState = {
  actionId: 'movePiece',
  actionType: 'movePiece',
  currentStepId: 'selectPiece',
  currentStepIndex: 0,
  totalSteps: 2,
  storedData: {},
  canCancel: true,
  requiresConfirmation: false,
  direction: 'Select a piece to move'
};

describe('MultiStepActionDisplay', () => {
  it('should render progress indicator for multi-step action', () => {
    const onCancel = vi.fn();
    const onConfirm = vi.fn();

    render(
      <MultiStepActionDisplay
        multiStepState={mockMultiStepState}
        onCancel={onCancel}
        onConfirm={onConfirm}
      />
    );

    // Check progress display
    expect(screen.getByText('Action in Progress')).toBeInTheDocument();
    expect(screen.getByText('Step 1 of 2')).toBeInTheDocument();
    expect(screen.getByText('50%')).toBeInTheDocument();
    expect(screen.getByText('Select a piece to move')).toBeInTheDocument();

    // Check cancel button
    const cancelButton = screen.getByLabelText('Cancel current action');
    expect(cancelButton).toBeInTheDocument();
    fireEvent.click(cancelButton);
    expect(onCancel).toHaveBeenCalled();
  });

  it('should render confirmation dialog when requiresConfirmation is true', () => {
    const confirmationState = {
      ...mockMultiStepState,
      currentStepIndex: 1,
      requiresConfirmation: true,
      confirmationPrompt: 'Move piece from A1 to B2?',
    };

    const onCancel = vi.fn();
    const onConfirm = vi.fn();

    render(
      <MultiStepActionDisplay
        multiStepState={confirmationState}
        onCancel={onCancel}
        onConfirm={onConfirm}
      />
    );

    // Check confirmation display
    expect(screen.getByText('Confirm Action')).toBeInTheDocument();
    expect(screen.getByText('Move piece from A1 to B2?')).toBeInTheDocument();
    expect(screen.getByText('Ready to complete')).toBeInTheDocument();

    // Check confirm button
    const confirmButton = screen.getByLabelText('Confirm and complete action');
    expect(confirmButton).toBeInTheDocument();
    fireEvent.click(confirmButton);
    expect(onConfirm).toHaveBeenCalled();
  });

  it('should show completed steps with checkmarks', () => {
    const advancedState = {
      ...mockMultiStepState,
      currentStepIndex: 1,
      storedData: { selectedPiece: 'piece_p1' },
    };

    render(
      <MultiStepActionDisplay
        multiStepState={advancedState}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );

    // Check that first step shows as completed
    const completedStep = screen.getByLabelText('Step 1 completed');
    expect(completedStep).toBeInTheDocument();
    expect(completedStep).toHaveClass('bg-blue-600');

    // Current step should be highlighted
    const currentStep = screen.getByLabelText('Step 2 current');
    expect(currentStep).toBeInTheDocument();
    expect(currentStep).toHaveClass('bg-blue-100');
  });

  it('should not show cancel button when canCancel is false', () => {
    const nonCancellableState = {
      ...mockMultiStepState,
      canCancel: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={nonCancellableState}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );

    const cancelButton = screen.queryByLabelText('Cancel current action');
    expect(cancelButton).not.toBeInTheDocument();
  });

  it('should return null when multiStepState is null', () => {
    const { container } = render(
      <MultiStepActionDisplay
        multiStepState={null}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );

    expect(container.firstChild).toBeNull();
  });

  it('should show correct progress percentage', () => {
    const threeStepState = {
      ...mockMultiStepState,
      currentStepIndex: 1,
      totalSteps: 3,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={threeStepState}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );

    // Step 2 of 3 should show 67%
    expect(screen.getByText('Step 2 of 3')).toBeInTheDocument();
    expect(screen.getByText('67%')).toBeInTheDocument();
  });

  it('should handle different action types with appropriate icons', () => {
    const cardActionState = {
      ...mockMultiStepState,
      actionType: 'askForCard',
      direction: 'Choose which rank to ask for'
    };

    render(
      <MultiStepActionDisplay
        multiStepState={cardActionState}
        onCancel={vi.fn()}
        onConfirm={vi.fn()}
      />
    );

    // Should show the card-specific direction
    expect(screen.getByText('Choose which rank to ask for')).toBeInTheDocument();
  });
});