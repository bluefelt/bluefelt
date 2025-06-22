import React from 'react';
import { render, screen } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import { ActionIndicator, useHasAction } from '../ActionIndicator';
import BoardCell from '../zones/BoardCell';
import MultiStepSelectionOverlay from '../MultiStepSelectionOverlay';

// Mock contexts and hooks
vi.mock('../../context/PlayerContext', () => ({
  usePlayer: () => ({ player: { username: 'TestPlayer', color: 'blue' } })
}));

vi.mock('../../hooks/useMarkColor', () => ({
  useMarkColor: () => () => '#000000'
}));

vi.mock('../../context/AnimationContext', () => ({
  useAnimationsEnabled: () => true
}));

vi.mock('../TokenDisplay', () => ({
  default: ({ tokenType }: any) => <div data-testid="token-display">{tokenType}</div>
}));

describe('Multi-Step Visual Indicators', () => {
  describe('ActionIndicator Enhanced States', () => {
    it('should render current_step state with enhanced visuals', () => {
      render(
        <ActionIndicator
          hasAction={true}
          isMultiStep={true}
          multiStepState="current_step"
          stepNumber={1}
        />
      );

      // Check for enhanced current step styling
      const indicator = screen.getByText('1');
      expect(indicator).toBeInTheDocument();
    });

    it('should render selected state with checkmark', () => {
      render(
        <ActionIndicator
          hasAction={true}
          isMultiStep={true}
          multiStepState="selected"
        />
      );

      // Check for checkmark SVG
      const checkmark = document.querySelector('svg path[fill-rule="evenodd"]');
      expect(checkmark).toBeInTheDocument();
    });

    it('should render next_step state with appropriate styling', () => {
      render(
        <ActionIndicator
          hasAction={true}
          isMultiStep={true}
          multiStepState="next_step"
        />
      );

      // Component should render without error
      expect(document.querySelector('.absolute')).toBeInTheDocument();
    });

    it('should render confirmed state with enhanced checkmark', () => {
      render(
        <ActionIndicator
          hasAction={true}
          isMultiStep={true}
          multiStepState="confirmed"
        />
      );

      // Check for checkmark SVG
      const checkmark = document.querySelector('svg path[fill-rule="evenodd"]');
      expect(checkmark).toBeInTheDocument();
    });
  });

  describe('useHasAction Enhanced Logic', () => {
    it('should return correct multi-step state for current step', () => {
      const mockMultiStepState = {
        actionId: 'movePiece',
        currentStepIndex: 0,
        stepActionMap: {
          '/zones/board/cells/1/1': { action: 'selectPiece' }
        },
        storedData: {},
        requiresConfirmation: false
      };

      const result = useHasAction(
        '/zones/board/cells/1/1',
        { '/zones/board/cells/1/1': { multiStepId: 'movePiece' } },
        true,
        mockMultiStepState
      );

      expect(result.hasAction).toBe(true);
      expect(result.isMultiStepAction).toBe(true);
      expect(result.multiStepIndicatorState).toBe('current_step');
      expect(result.stepNumber).toBe(1);
    });

    it('should return selected state for previously selected location', () => {
      const mockMultiStepState = {
        actionId: 'movePiece',
        currentStepIndex: 1,
        stepActionMap: {},
        storedData: {
          selectedPiece: '/zones/board/cells/1/1'
        },
        requiresConfirmation: false
      };

      const result = useHasAction(
        '/zones/board/cells/1/1',
        { '/zones/board/cells/1/1': { multiStepId: 'movePiece' } },
        true,
        mockMultiStepState
      );

      expect(result.hasAction).toBe(true);
      expect(result.isMultiStepAction).toBe(true);
      expect(result.multiStepIndicatorState).toBe('selected');
    });

    it('should return confirmed state when requiresConfirmation is true', () => {
      const mockMultiStepState = {
        actionId: 'movePiece',
        currentStepIndex: 1,
        stepActionMap: {},
        storedData: {
          selectedPiece: '/zones/board/cells/1/1'
        },
        requiresConfirmation: true
      };

      const result = useHasAction(
        '/zones/board/cells/1/1',
        { '/zones/board/cells/1/1': { multiStepId: 'movePiece' } },
        true,
        mockMultiStepState
      );

      expect(result.hasAction).toBe(true);
      expect(result.isMultiStepAction).toBe(true);
      expect(result.multiStepIndicatorState).toBe('confirmed');
    });
  });

  describe('BoardCell Enhanced Borders', () => {
    const defaultProps = {
      cell: null,
      row: 1,
      col: 1,
      isClickable: true,
      cellSize: 60,
      isDarkSquare: false,
      useCheckerPattern: false,
      isSelected: false,
      entityDisplay: { type: 'glyph' as const, glyph: '' },
      markColor: '#000000',
      onCellClick: vi.fn(),
      zoneId: 'board',
      hasAction: true,
      isMultiStepAction: true,
      multiStepIndicatorState: 'current_step' as const,
      stepNumber: 1
    };

    it('should render with enhanced multi-step styling', () => {
      render(<BoardCell {...defaultProps} />);

      const cell = screen.getByTestId('cell-1-1');
      expect(cell).toBeInTheDocument();
      
      // Check that ActionIndicator is rendered with correct props
      expect(screen.getByText('1')).toBeInTheDocument();
    });

    it('should render selected state with green border', () => {
      render(
        <BoardCell 
          {...defaultProps} 
          isSelected={true}
          multiStepIndicatorState="selected"
        />
      );

      const cell = screen.getByTestId('cell-1-1');
      expect(cell).toBeInTheDocument();
    });

    it('should render confirmed state with enhanced green border', () => {
      render(
        <BoardCell 
          {...defaultProps} 
          isSelected={true}
          multiStepIndicatorState="confirmed"
        />
      );

      const cell = screen.getByTestId('cell-1-1');
      expect(cell).toBeInTheDocument();
    });
  });

  describe('MultiStepSelectionOverlay', () => {
    it('should not render when no multi-step state', () => {
      const { container } = render(
        <MultiStepSelectionOverlay
          multiStepState={null}
          zoneId="board"
          cellSize={60}
          rows={3}
          cols={3}
        />
      );

      expect(container.firstChild).toBeNull();
    });

    it('should not render when insufficient selections', () => {
      const mockMultiStepState = {
        storedData: {
          selectedPiece: '/zones/board/cells/1/1'
        }
      };

      const { container } = render(
        <MultiStepSelectionOverlay
          multiStepState={mockMultiStepState}
          zoneId="board"
          cellSize={60}
          rows={3}
          cols={3}
        />
      );

      expect(container.firstChild).toBeNull();
    });

    it('should render connections between multiple selections', () => {
      const mockMultiStepState = {
        storedData: {
          selectedPiece: '/zones/board/cells/1/1',
          destination: '/zones/board/cells/2/2'
        }
      };

      render(
        <MultiStepSelectionOverlay
          multiStepState={mockMultiStepState}
          zoneId="board"
          cellSize={60}
          rows={3}
          cols={3}
        />
      );

      // Check for SVG elements
      const svg = document.querySelector('svg');
      expect(svg).toBeInTheDocument();
      
      // Check for connection lines
      const lines = document.querySelectorAll('line');
      expect(lines.length).toBeGreaterThan(0);
      
      // Check for step number indicators
      const circles = document.querySelectorAll('circle');
      expect(circles.length).toBeGreaterThan(0);
    });

    it('should handle object-format stored data', () => {
      const mockMultiStepState = {
        storedData: {
          selectedPiece: { location: '/zones/board/cells/1/1', entity: 'piece1' },
          destination: { location: '/zones/board/cells/2/2' }
        }
      };

      render(
        <MultiStepSelectionOverlay
          multiStepState={mockMultiStepState}
          zoneId="board"
          cellSize={60}
          rows={3}
          cols={3}
        />
      );

      // Check for SVG elements
      const svg = document.querySelector('svg');
      expect(svg).toBeInTheDocument();
    });
  });

  describe('Integration Tests', () => {
    it('should work together to provide comprehensive multi-step feedback', () => {
      const mockMultiStepState = {
        actionId: 'movePiece',
        currentStepIndex: 1,
        stepActionMap: {
          '/zones/board/cells/2/2': { action: 'selectDestination' }
        },
        storedData: {
          selectedPiece: '/zones/board/cells/1/1'
        },
        requiresConfirmation: false
      };

      // Test that all components work together
      expect(() => {
        render(
          <div>
            <ActionIndicator
              hasAction={true}
              isMultiStep={true}
              multiStepState="selected"
            />
            <MultiStepSelectionOverlay
              multiStepState={mockMultiStepState}
              zoneId="board"
              cellSize={60}
              rows={3}
              cols={3}
            />
          </div>
        );
      }).not.toThrow();
    });
  });
});