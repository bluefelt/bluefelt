import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import Board from '../zones/Board';
import { PlayerProvider } from '../../context/PlayerContext';

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
      { id: 'board', type: 'grid', gridDimensions: { rows: 3, cols: 3 } }
    ],
    playerNames: ['player1', 'player2'],
    actionMap: undefined // Board component uses undefined to enable all empty cells
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Basic Rendering', () => {
    it('should render the board component', () => {
      render(
        <PlayerProvider>
          <Board {...defaultProps} />
        </PlayerProvider>
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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
      );

      // Should render nothing when no zones
      expect(screen.queryByText('Board')).not.toBeInTheDocument();
    });
  });

  describe('Click Handling', () => {
    it('should handle cell clicks when it is my turn', () => {
      const { container } = render(
        <PlayerProvider>
          <Board {...defaultProps} />
        </PlayerProvider>
      );

      // Find clickable cells (empty cells when it's my turn)
      const cells = container.querySelectorAll('.cursor-pointer');
      expect(cells.length).toBeGreaterThan(0);

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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
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
          { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } },
          { id: 'secondBoard', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
        ]
      };

      render(
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
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
          board: null
        }
      };

      render(
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
      );

      // Should not crash
      expect(screen.queryByText('Board')).not.toBeInTheDocument();
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
        <PlayerProvider>
          <Board {...props} />
        </PlayerProvider>
      );

      // Should only render the board
      expect(screen.getByText('Board')).toBeInTheDocument();
      expect(screen.queryByText('Marks')).not.toBeInTheDocument();
      expect(screen.queryByText('Deck')).not.toBeInTheDocument();
    });
  });
});