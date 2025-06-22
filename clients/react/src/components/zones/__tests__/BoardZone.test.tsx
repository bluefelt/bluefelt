import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import BoardZone from '../BoardZone';
import { AnimationProvider } from '../../../context/AnimationContext';
import { ReactNode } from 'react';

// Mock the PlayerContext
vi.mock('../../../context/PlayerContext', () => ({
  usePlayer: () => ({
    player: { username: 'p1', color: '#ff0000' }
  })
}));

// Mock the useMarkColor hook
vi.mock('../../../hooks/useMarkColor', () => ({
  useMarkColor: () => () => '#ff0000'
}));

// Mock ActionIndicator hook and component
vi.mock('../../ActionIndicator', () => ({
  useHasAction: () => ({ hasAction: false, isMultiStepAction: false }),
  ActionIndicator: ({ hasAction, isMultiStep }: any) => hasAction ? <div>Action Available</div> : null
}));

// Test wrapper
const TestWrapper = ({ children }: { children: ReactNode }) => (
  <AnimationProvider>
    {children}
  </AnimationProvider>
);

describe('BoardZone Selection State', () => {
  const defaultProps = {
    zoneId: 'board',
    boardData: [
      [null, null, null],
      [null, 'piece_p1', null],
      [null, null, null]
    ],
    isMyTurn: true,
    onCellClick: vi.fn(),
    entityDefinitions: [
      { id: 'piece_p1', ui: { glyph: 'X' } }
    ],
    playerNames: ['Alice', 'Bob'],
    actionMap: {
      '/zones/board/cells/0/0': { action: 'selectPiece' },
      '/zones/board/cells/1/1': { action: 'selectPiece' }
    }
  };

  it('should highlight selected cell with old format selection', () => {
    const selection = {
      zone: 'board',
      row: 1,
      col: 1
    };

    render(<BoardZone {...defaultProps} selection={selection} />, { wrapper: TestWrapper });

    // Find the cell that should be selected
    const selectedCell = screen.getByTestId('cell-1-1');
    expect(selectedCell).toHaveClass('border-yellow-400', 'border-2');
  });

  it('should highlight selected cell with new server format selection', () => {
    const selection = {
      p1: {
        location: '/zones/board/cells/1/1',
        entity: { entity: 'piece_p1' }
      }
    };

    render(<BoardZone {...defaultProps} selection={selection} />, { wrapper: TestWrapper });

    // Find the cell that should be selected
    const selectedCell = screen.getByTestId('cell-1-1');
    expect(selectedCell).toHaveClass('border-yellow-400', 'border-2');
  });

  it('should highlight selected cell with alternate path format', () => {
    const selection = {
      p1: {
        location: '/zones/board/1/1',  // Without "cells" in path
        entity: { entity: 'piece_p1' }
      }
    };

    render(<BoardZone {...defaultProps} selection={selection} />, { wrapper: TestWrapper });

    // Find the cell that should be selected
    const selectedCell = screen.getByTestId('cell-1-1');
    expect(selectedCell).toHaveClass('border-yellow-400', 'border-2');
  });

  it('should not highlight cells when no selection', () => {
    render(<BoardZone {...defaultProps} selection={null} />, { wrapper: TestWrapper });

    // All cells should have normal borders
    const cells = screen.getAllByTestId(/cell-\d-\d/);
    cells.forEach(cell => {
      expect(cell).toHaveClass('border-gray-700');
      expect(cell).not.toHaveClass('border-yellow-400');
    });
  });

  it('should handle multiple players in selection object', () => {
    const selection = {
      p1: {
        location: '/zones/board/cells/0/0',
        entity: { entity: 'piece_p1' }
      },
      p2: {
        location: '/zones/board/cells/2/2', 
        entity: { entity: 'piece_p2' }
      }
    };

    render(<BoardZone {...defaultProps} selection={selection} />, { wrapper: TestWrapper });

    // Only p1's selection should be highlighted (since we're checking all players)
    const selectedCell1 = screen.getByTestId('cell-0-0');
    expect(selectedCell1).toHaveClass('border-yellow-400', 'border-2');

    const selectedCell2 = screen.getByTestId('cell-2-2');
    expect(selectedCell2).toHaveClass('border-yellow-400', 'border-2');
  });

  it('should not highlight when selection location does not match any cell', () => {
    const selection = {
      p1: {
        location: '/zones/board/cells/5/5', // Out of bounds
        entity: { entity: 'piece_p1' }
      }
    };

    render(<BoardZone {...defaultProps} selection={selection} />, { wrapper: TestWrapper });

    // No cells should be highlighted
    const cells = screen.getAllByTestId(/cell-\d-\d/);
    cells.forEach(cell => {
      expect(cell).toHaveClass('border-gray-700');
      expect(cell).not.toHaveClass('border-yellow-400');
    });
  });
});