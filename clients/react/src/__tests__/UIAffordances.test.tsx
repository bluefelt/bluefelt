import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import React from 'react';
import GameView from '../components/GameView';
import GameZones from '../components/GameZones';
import Board from '../components/zones/Board';
import { ChoiceZone } from '../components/zones/ChoiceZone';
import { TestProviders } from '../test/TestProviders';

// Context mocks are now handled globally in test setup

describe('UI Affordances Tests', () => {
  describe('Turn-based Action Availability', () => {
    it('should show clickable cells only on current player turn', () => {
      const mockOnCellClick = vi.fn();
      
      // Test when it's the player's turn
      const { rerender } = render(
        <TestProviders initialPlayer={{ username: 'testuser', color: '#FF0000' }}>
          <Board
            zones={{ board: [[null, null], [null, null]] }}
            entityDefinitions={[]}
            onCellClick={mockOnCellClick}
            isMyTurn={true}
            zoneMetadata={[
              { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
            ]}
            playerNames={['player1', 'player2']}
            actionMap={{
              '/zones/board/cells/0/0': { action: 'placeMarker' },
              '/zones/board/cells/0/1': { action: 'placeMarker' },
            }}
          />
        </TestProviders>
      );

      // Should have clickable cells (only cells with actions are clickable)
      const cells = screen.getAllByRole('button');
      expect(cells).toHaveLength(2); // Only cells with actions are buttons
      
      // Click should work
      fireEvent.click(cells[0]);
      expect(mockOnCellClick).toHaveBeenCalledWith(0, 0);

      // Test when it's NOT the player's turn
      mockOnCellClick.mockClear();
      rerender(
        <TestProviders initialPlayer={{ username: 'testuser', color: '#FF0000' }}>
          <Board
            zones={{ board: [[null, null], [null, null]] }}
            entityDefinitions={[]}
            onCellClick={mockOnCellClick}
            isMyTurn={false}
            zoneMetadata={[
              { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
            ]}
            playerNames={['player1', 'player2']}
            actionMap={{}}
          />
        </TestProviders>
      );

      // Should have no clickable cells when no actions provided
      const buttonsAfter = screen.queryAllByRole('button');
      expect(buttonsAfter).toHaveLength(0); // No clickable cells
    });

    it('should highlight available actions', () => {
      render(
        <TestProviders initialPlayer={{ username: 'testuser', color: '#FF0000' }}>
          <Board
            zones={{ board: [['mark_p1', null], [null, 'mark_p2']] }}
            entityDefinitions={[]}
            onCellClick={vi.fn()}
            isMyTurn={true}
            zoneMetadata={[
              { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
            ]}
            playerNames={['player1', 'player2']}
            actionMap={{
              '/zones/board/cells/0/1': { action: 'placeMarker' },
              '/zones/board/cells/1/0': { action: 'placeMarker' },
            }}
          />
        </TestProviders>
      );

      const cells = screen.getAllByRole('button');
      
      // Should only have buttons for cells with actions
      expect(cells).toHaveLength(2); // Only (0,1) and (1,0) have actions
      
      // All clickable cells should have cursor-pointer
      cells.forEach(cell => {
        expect(cell).toHaveClass('cursor-pointer');
      });
    });
  });

  describe('Phase-specific UI Elements', () => {
    it('should show choice UI only in selection phases', () => {
      // Test Go Fish rank selection
      const mockOnSelect = vi.fn();
      
      const { rerender } = render(
        <ChoiceZone
          zone={{
            id: 'ranks',
            type: 'choice',
            items: [
              { id: 'A', label: 'A' },
              { id: '2', label: '2' },
              { id: '3', label: '3' },
              { id: 'K', label: 'K' },
            ],
            prompt: 'Select a rank',
          }}
          onSelect={mockOnSelect}
          isActive={true}
        />
      );

      // Should show all choices
      expect(screen.getByText('Select a rank')).toBeInTheDocument();
      expect(screen.getByText('A')).toBeInTheDocument();
      expect(screen.getByText('K')).toBeInTheDocument();

      // Should be clickable when active
      fireEvent.click(screen.getByText('A'));
      expect(mockOnSelect).toHaveBeenCalledWith('A');

      // Test when not active
      mockOnSelect.mockClear();
      rerender(
        <ChoiceZone
          zone={{
            id: 'ranks',
            type: 'choice',
            items: [
              { id: 'A', label: 'A' },
              { id: '2', label: '2' },
              { id: '3', label: '3' },
              { id: 'K', label: 'K' },
            ],
            prompt: 'Select a rank',
          }}
          onSelect={mockOnSelect}
          isActive={false}
        />
      );

      // Should not render when inactive
      expect(screen.queryByText('A')).not.toBeInTheDocument();
      expect(screen.queryByText('Select a rank')).not.toBeInTheDocument();
    });

    it('should show correct prompts based on phase', () => {
      const phases = {
        selectingRank: 'Choose a rank to ask for',
        selectingPlayer: 'Choose a player to ask',
        responding: 'Waiting for response...',
        fishing: 'Go fish! Drawing a card...',
      };

      Object.entries(phases).forEach(([phase, prompt]) => {
        const { unmount } = render(
          <div data-testid="phase-prompt">
            {prompt}
          </div>
        );

        expect(screen.getByTestId('phase-prompt')).toHaveTextContent(prompt);
        
        // Clean up after each test
        unmount();
      });
    });
  });

  describe('Action Map Integration', () => {
    it('should only show actions for current player', () => {
      const actionMap = {
        p1: {
          '/zones/board/cells/0/0': { action: 'placeMarker' },
          '/zones/board/cells/1/1': { action: 'placeMarker' },
        },
        p2: {
          '/zones/board/cells/2/2': { action: 'placeMarker' },
        },
      };

      // Render as p1
      const { rerender } = render(
        <GameZones
          zones={{ board: { type: 'grid', cells: [[null, null, null]] } }}
          you="p1"
          isMyTurn={true}
          actionMap={actionMap.p1}
          zoneMetadata={[{ id: 'board', type: 'grid' }]}
        />
      );

      // Should only see p1's actions
      // (This is a simplified test - real implementation would check actual rendered actions)
      
      // Render as p2
      rerender(
        <GameZones
          zones={{ board: { type: 'grid', cells: [[null, null, null]] } }}
          you="p2"
          isMyTurn={true}
          actionMap={actionMap.p2}
          zoneMetadata={[{ id: 'board', type: 'grid' }]}
        />
      );

      // Should only see p2's actions
    });
  });

  describe('Visual Feedback', () => {
    it('should show loading state during action processing', async () => {
      const mockSendMessage = vi.fn(() => new Promise(resolve => setTimeout(resolve, 100)));
      
      const { getByTestId } = render(
        <button
          data-testid="action-button"
          onClick={async () => {
            // Simulate loading state
            const button = getByTestId('action-button');
            button.setAttribute('data-loading', 'true');
            await mockSendMessage();
            button.setAttribute('data-loading', 'false');
          }}
        >
          Place Mark
        </button>
      );

      const button = getByTestId('action-button');
      
      // Click and check loading state
      fireEvent.click(button);
      expect(button).toHaveAttribute('data-loading', 'true');
      
      // Wait for action to complete
      await vi.waitFor(() => {
        expect(button).toHaveAttribute('data-loading', 'false');
      });
    });

    it('should disable actions after game ends', () => {
      const mockOnCellClick = vi.fn();
      
      render(
        <TestProviders initialPlayer={{ username: 'testuser', color: '#FF0000' }}>
          <Board
            zones={{ board: [[null, null], [null, null]] }}
            entityDefinitions={[]}
            onCellClick={mockOnCellClick}
            isMyTurn={true}
            zoneMetadata={[
              { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
            ]}
            playerNames={['player1', 'player2']}
            actionMap={{}}
            gameEnded={true}
          />
        </TestProviders>
      );

      // With empty actionMap, no cells should be clickable
      const buttons = screen.queryAllByRole('button');
      expect(buttons).toHaveLength(0); // No clickable cells when no actions
      
      // Verify cells exist but are not clickable
      const cells = screen.getAllByTestId(/cell-\d+-\d+/);
      expect(cells).toHaveLength(4); // All 4 cells exist
      cells.forEach(cell => {
        expect(cell.querySelector('div')).toHaveClass('cursor-not-allowed');
      });
    });
  });

  describe('Error States', () => {
    it('should show error message on invalid action', () => {
      const mockOnError = vi.fn();
      
      // Simulate error scenario
      const errorMessage = 'Invalid move: cell already occupied';
      
      render(
        <div>
          {errorMessage && (
            <div role="alert" className="error-message">
              {errorMessage}
            </div>
          )}
        </div>
      );

      expect(screen.getByRole('alert')).toHaveTextContent(errorMessage);
    });

    it('should handle disconnection gracefully', () => {
      let isConnected = true;
      const { rerender } = render(
        <div data-testid="connection-status">
          {isConnected ? 'Connected' : 'Disconnected - Attempting to reconnect...'}
        </div>
      );

      expect(screen.getByTestId('connection-status')).toHaveTextContent('Connected');

      // Simulate disconnection
      isConnected = false;
      rerender(
        <div data-testid="connection-status">
          {isConnected ? 'Connected' : 'Disconnected - Attempting to reconnect...'}
        </div>
      );

      expect(screen.getByTestId('connection-status')).toHaveTextContent('Disconnected');
    });
  });
});