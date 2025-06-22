import React from 'react';
import { render, screen, fireEvent } from '@testing-library/react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { MultiStepActionDisplay } from '../MultiStepActionDisplay';

// Mock the UI components
vi.mock('../ui/Button', () => ({
  Button: ({ children, onClick, ...props }: any) => (
    <button onClick={onClick} {...props}>
      {children}
    </button>
  ),
}));

vi.mock('../ui/Card', () => ({
  Card: ({ children, ...props }: any) => (
    <div {...props}>
      {children}
    </div>
  ),
}));

describe('MultiStepActionDisplay', () => {
  const mockOnCancel = vi.fn();
  const mockOnConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('should not render when multiStepState is null', () => {
    render(
      <MultiStepActionDisplay
        multiStepState={null}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    expect(screen.queryByText('Action in Progress')).not.toBeInTheDocument();
  });

  it('should render progress display for active multi-step action', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'selectPiece',
      currentStepIndex: 0,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    expect(screen.getByText('Action in Progress')).toBeInTheDocument();
    expect(screen.getByText('Step 1 of 2')).toBeInTheDocument();
    expect(screen.getByText('50%')).toBeInTheDocument();
  });

  it('should render confirmation dialog when requiresConfirmation is true', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'confirm',
      currentStepIndex: 2,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: true,
      confirmationPrompt: 'Move piece from A1 to B2?',
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    expect(screen.getByText('Confirm Action')).toBeInTheDocument();
    expect(screen.getByText('Move piece from A1 to B2?')).toBeInTheDocument();
    expect(screen.getByText('Confirm')).toBeInTheDocument();
  });

  it('should call onCancel when cancel button is clicked', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'selectPiece',
      currentStepIndex: 0,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    fireEvent.click(screen.getByText('Cancel'));
    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it('should call onConfirm when confirm button is clicked', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'confirm',
      currentStepIndex: 2,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: true,
      confirmationPrompt: 'Move piece from A1 to B2?',
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    fireEvent.click(screen.getByText('Confirm'));
    expect(mockOnConfirm).toHaveBeenCalledTimes(1);
  });

  it('should not show cancel button when canCancel is false', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'selectPiece',
      currentStepIndex: 0,
      totalSteps: 2,
      storedData: {},
      canCancel: false,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    expect(screen.queryByText('Cancel')).not.toBeInTheDocument();
  });

  it('should display correct step instructions for different action types', () => {
    const multiStepState = {
      actionId: 'askForCard',
      actionType: 'askForCard',
      currentStepId: 'selectRank',
      currentStepIndex: 0,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    expect(screen.getByText('Choose which rank to ask for')).toBeInTheDocument();
  });

  it('should show progress indicators with correct completion status', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'selectDestination',
      currentStepIndex: 1,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    // Check that step 1 is completed (should have checkmark)
    expect(screen.getByLabelText('Step 1 completed')).toBeInTheDocument();
    // Check that step 2 is current
    expect(screen.getByLabelText('Step 2 current')).toBeInTheDocument();
  });

  it('should have proper accessibility attributes', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'selectPiece',
      currentStepIndex: 0,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: false,
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    // Check for proper ARIA attributes
    expect(screen.getByRole('status')).toBeInTheDocument();
    expect(screen.getByLabelText('Step 1 of 2')).toBeInTheDocument();
    expect(screen.getByLabelText('Cancel current action')).toBeInTheDocument();
  });

  it('should have proper accessibility attributes for confirmation dialog', () => {
    const multiStepState = {
      actionId: 'movePiece',
      actionType: 'movePiece',
      currentStepId: 'confirm',
      currentStepIndex: 2,
      totalSteps: 2,
      storedData: {},
      canCancel: true,
      requiresConfirmation: true,
      confirmationPrompt: 'Move piece from A1 to B2?',
    };

    render(
      <MultiStepActionDisplay
        multiStepState={multiStepState}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
      />
    );

    // Check for dialog accessibility
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.getByLabelText('Action confirmation buttons')).toBeInTheDocument();
    expect(screen.getByLabelText('Cancel action')).toBeInTheDocument();
    expect(screen.getByLabelText('Confirm and complete action')).toBeInTheDocument();
  });
});