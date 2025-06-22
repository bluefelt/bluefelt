import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import Board from '../zones/Board';
import { PlayerProvider } from '../../context/PlayerContext';
import { AnimationProvider } from '../../context/AnimationContext';
import { PlayerPreferencesProvider } from '../../context/PlayerPreferencesContext';
import { ReactNode } from 'react';

// Mock the PlayerContext
vi.mock('../../context/PlayerContext', async () => {
  const actual = await vi.importActual('../../context/PlayerContext');
  return {
    ...actual,
    usePlayer: vi.fn(() => ({
      player: { username: 'testuser', color: '#FF0000' }
    }))
  };
});

// Mock the hooks
vi.mock('../../hooks/useMarkColor', () => ({
  useMarkColor: () => () => '#FF0000'
}));

// Mock ActionIndicator hook and component
vi.mock('../ActionIndicator', () => ({
  useHasAction: () => ({ hasAction: false, isMultiStepAction: false }),
  ActionIndicator: ({ hasAction, isMultiStep }: any) => hasAction ? <div>Action Available</div> : null
}));

// Test wrapper to provide all necessary contexts
const TestWrapper = ({ children }: { children: ReactNode }) => (
  <PlayerPreferencesProvider>
    <AnimationProvider>
      <PlayerProvider>
        {children}
      </PlayerProvider>
    </AnimationProvider>
  </PlayerPreferencesProvider>
);

describe('Board Component', () => {
  const mockOnCellClick = vi.fn();
  
  const defaultProps = {
    zones: {
      board: [
        [null, null, null],
        [null, null, null],
        [null, null, null]
      ]
    },
    entityDefinitions: [
      { id: 'mark_p1', props: { value: 'p1' }, ui: { display: 'X' } },
      { id: 'mark_p2', props: { value: 'p2' }, ui: { display: 'O' } }
    ],
    onCellClick: mockOnCellClick,
    isMyTurn: true,
    zoneMetadata: [
      { id: 'board', renderType: 'grid', visibility: 'all', gridDimensions: { rows: 3, cols: 3 } }
    ],
    playerNames: ['player1', 'player2'],
    actionMap: {} // Empty action map - server authority means no actions without explicit map
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('should render the board component', () => {
      render(
        <Board {...defaultProps} />,
        { wrapper: TestWrapper }
      );

      expect(screen.getByText('Board')).toBeInTheDocument();
    });

    it('should render entities correctly', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: [
            ['mark_p1', null, 'mark_p2'],
            [null, 'mark_p1', null],
            ['mark_p2', null, null]
          ]
        }
      };

      render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Check for X and O displays
      const xMarks = screen.getAllByText('X');
      const oMarks = screen.getAllByText('O');
      expect(xMarks).toHaveLength(2);
      expect(oMarks).toHaveLength(2);
    });

    it('should handle empty zones gracefully', () => {
      const props = {
        ...defaultProps,
        zones: {}
      };

      render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Should render nothing when no zones
      const boardElement = screen.queryByTestId('board-zone');
      expect(boardElement).not.toBeInTheDocument();
    });
  });

  describe('Click Handling', () => {
    it('should handle cell clicks when it is my turn', () => {
      // For server authority, we need to provide an action map
      const propsWithActions = {
        ...defaultProps,
        actionMap: {
          '/zones/board/cells/0/0': { action: 'place', direction: 'Click to place' },
          '/zones/board/cells/1/1': { action: 'place', direction: 'Click to place' }
        }
      };
      
      const { container } = render(
        <Board {...propsWithActions} />,
        { wrapper: TestWrapper }
      );

      // Find clickable cells based on action map
      const cells = container.querySelectorAll('.cursor-pointer');
      expect(cells.length).toBe(2); // Should match action map entries

      // Click the first clickable cell
      fireEvent.click(cells[0]);
      expect(mockOnCellClick).toHaveBeenCalled();
    });

    it('should not allow clicks when not my turn', () => {
      const props = {
        ...defaultProps,
        isMyTurn: false
      };

      const { container } = render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // All cells should have cursor-not-allowed
      const notAllowedCells = container.querySelectorAll('.cursor-not-allowed');
      expect(notAllowedCells.length).toBe(9);

      // Try clicking a cell
      fireEvent.click(notAllowedCells[0]);
      expect(mockOnCellClick).not.toHaveBeenCalled();
    });
  });

  describe('Action Map Support', () => {
    it('should only enable cells with valid actions when actionMap is provided', () => {
      const props = {
        ...defaultProps,
        actionMap: {
          '/zones/board/0/0': { action: 'place', direction: 'Click to place' },
          '/zones/board/1/1': { action: 'place', direction: 'Click to place' }
        }
      };

      const { container } = render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Should have exactly 2 clickable cells
      const clickableCells = container.querySelectorAll('.cursor-pointer');
      expect(clickableCells.length).toBe(2);
    });

    it('should support both path formats in actionMap', () => {
      const props = {
        ...defaultProps,
        actionMap: {
          '/zones/board/cells/0/0': { action: 'place' },
          '/zones/board/1/1': { action: 'place' }
        }
      };

      const { container } = render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Both formats should work
      const clickableCells = container.querySelectorAll('.cursor-pointer');
      expect(clickableCells.length).toBe(2);
    });
  });

  describe('Multiple Zones', () => {
    it('should render multiple grid zones', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: [[null, null], [null, null]],
          secondBoard: [[null, null], [null, null]]
        },
        zoneMetadata: [
          { id: 'board', renderType: 'grid', visibility: 'all', gridDimensions: { rows: 2, cols: 2 } },
          { id: 'secondBoard', renderType: 'grid', visibility: 'all', gridDimensions: { rows: 2, cols: 2 } }
        ]
      };

      render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Should render both zone titles
      expect(screen.getByText('Board')).toBeInTheDocument();
      expect(screen.getByText('secondBoard')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle malformed zones gracefully', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: null as any
        }
      };

      render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Should not crash and should not render board zone
      const boardElement = screen.queryByTestId('board-zone');
      expect(boardElement).not.toBeInTheDocument();
    });

    it('should skip non-grid zones', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: [[null, null], [null, null]],
          marks: { p1: [], p2: [] }, // Not a 2D array
          deck: ['card1', 'card2'] // 1D array
        }
      };

      render(
        <Board {...props} />,
        { wrapper: TestWrapper }
      );

      // Should only render the board zone
      const boardZone = screen.getByTestId('board-zone');
      expect(boardZone).toBeInTheDocument();
      expect(screen.getByText('Board')).toBeInTheDocument();
      
      // Should not render non-grid zones
      const marksZone = screen.queryByTestId('marks-zone');
      const deckZone = screen.queryByTestId('deck-zone');
      expect(marksZone).not.toBeInTheDocument();
      expect(deckZone).not.toBeInTheDocument();
    });
  });
});