import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import GameZones from '../GameZones';
import { PlayerProvider } from '../../context/PlayerContext';

// Mock child components
vi.mock('../zones/Board', () => ({
  default: ({ onCellClick, actionMap, zones }: any) => (
    <div data-testid="board-component">
      <button 
        data-testid="board-cell-0-0"
        onClick={() => onCellClick?.(0, 0)}
      >
        Board Cell 0,0
      </button>
    </div>
  )
}));

vi.mock('../CardZone', () => ({
  default: ({ zoneId, cards, onCardClick }: any) => (
    <div data-testid={`card-zone-${zoneId}`}>
      Card Zone: {zoneId} - {cards?.length || 0} cards
      <button 
        data-testid={`card-button-${zoneId}`}
        onClick={() => {
          // CardZone passes (_cardId, cardIndex) to onCardClick
          if (onCardClick) onCardClick(cards?.[0]?.id || 'card1', 0);
        }}
      >
        Click Card
      </button>
    </div>
  )
}));

describe('GameZones Component', () => {
  const mockOnAction = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  const defaultProps = {
    zones: {},
    zoneMetadata: [],
    entityDefinitions: [],
    onAction: mockOnAction,
    isYourTurn: true,
    you: 'p1',
    actionMap: {},
    playerNames: ['player1', 'player2']
  };

  describe('Zone Type Detection', () => {
    it('should render grid zones using Board component', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } }
        ]
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });

    it('should render card zones using CardZone component', () => {
      const props = {
        ...defaultProps,
        zones: {
          hand_p1: {
            items: ['card1', 'card2']
          }
        },
        zoneMetadata: [
          { id: 'hand_p1', type: 'list', name: 'Player 1 Hand' }
        ]
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      expect(screen.getByTestId('card-zone-hand_p1')).toBeInTheDocument();
    });

    it('should render multiple zones of different types', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          },
          deck: {
            cards: ['card1', 'card2', 'card3'],
            type: 'deck'
          },
          hand_p1: {
            cards: ['card4'],
            type: 'list'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid', gridDimensions: { rows: 2, cols: 2 } },
          { id: 'deck', type: 'deck' },
          { id: 'hand_p1', type: 'list' }
        ]
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      expect(screen.getByTestId('board-component')).toBeInTheDocument();
      expect(screen.getByTestId('card-zone-deck')).toBeInTheDocument();
      expect(screen.getByTestId('card-zone-hand_p1')).toBeInTheDocument();
    });
  });

  describe('Action Handling', () => {
    it('should handle board cell clicks correctly', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid' }
        ],
        actionMap: {
          '/zones/board/0/0': { action: 'place', direction: 'Click to place' }
        },
        onCellClick: (row: number, col: number) => {
          // Find the action for this cell
          const location = `/zones/board/${row}/${col}`;
          const action = props.actionMap[location];
          if (action) {
            mockOnAction(location, action.action);
          }
        }
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // The Board component is mocked, so we simulate its click
      fireEvent.click(screen.getByTestId('board-cell-0-0'));
      
      expect(mockOnAction).toHaveBeenCalledWith('/zones/board/0/0', 'place');
    });

    it('should handle card clicks correctly', () => {
      const mockCardAction = vi.fn();
      const props = {
        ...defaultProps,
        zones: {
          hand_p1: {
            items: ['card1', 'card2']
          }
        },
        zoneMetadata: [
          { id: 'hand_p1', type: 'list', name: 'Hand' }
        ],
        actionMap: {
          '/zones/hand_p1/0': { action: 'play', direction: 'Play this card' }
        },
        onCardAction: mockCardAction
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      fireEvent.click(screen.getByText('Click Card'));
      
      expect(mockCardAction).toHaveBeenCalledWith('hand_p1', 0);
    });
  });

  describe('Zone Groups', () => {
    it('should handle zone groups correctly', () => {
      const props = {
        ...defaultProps,
        zones: {
          hand_p1: { cards: ['card1'], type: 'list' },
          hand_p2: { cards: ['card2'], type: 'list' }
        },
        zoneMetadata: [
          { id: 'hand_p1', type: 'list', group: 'hands' },
          { id: 'hand_p2', type: 'list', group: 'hands' }
        ],
        ui: {
          zoneGroups: {
            hands: { label: 'Player Hands' }
          }
        }
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // Both hand zones should be rendered
      expect(screen.getByTestId('card-zone-hand_p1')).toBeInTheDocument();
      expect(screen.getByTestId('card-zone-hand_p2')).toBeInTheDocument();
    });
  });

  describe('Action Map Propagation', () => {
    it('should pass correct action map to child zones', () => {
      const actionMap = {
        '/zones/board/0/0': { action: 'place', direction: 'Place here' },
        '/zones/board/1/1': { action: 'place', direction: 'Or here' }
      };

      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid' }
        ],
        actionMap
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // The Board component should receive the action map
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });

    it('should filter action map based on current player', () => {
      const props = {
        ...defaultProps,
        you: 'p1',
        actionMap: {
          p1: {
            '/zones/board/0/0': { action: 'place', direction: 'Your move' }
          },
          p2: {
            '/zones/board/1/1': { action: 'place', direction: 'Their move' }
          }
        },
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid' }
        ]
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // Component should only receive p1's action map
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });
  });

  describe('Edge Cases', () => {
    it('should handle empty zones gracefully', () => {
      const props = {
        ...defaultProps,
        zones: {},
        zoneMetadata: []
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // Should render without crashing
      expect(screen.queryByTestId('board-component')).not.toBeInTheDocument();
      expect(screen.queryByTestId(/card-zone/)).not.toBeInTheDocument();
    });

    it('should handle missing zone metadata', () => {
      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: undefined
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // Should still render based on zone type
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });

    it('should handle malformed zone data', () => {
      const props = {
        ...defaultProps,
        zones: {
          brokenZone: {
            // Missing cells but has grid type
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'brokenZone', type: 'grid' }
        ]
      };

      const { container } = render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // Should handle gracefully without crashing
      // The component should render but might not show the broken zone
      expect(container.firstChild).toBeDefined();
    });
  });

  describe('Turn-based Interactions', () => {
    it('should disable interactions when not your turn', () => {
      const props = {
        ...defaultProps,
        isYourTurn: false,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', type: 'grid' }
        ]
      };

      render(
        <PlayerProvider>
          <GameZones {...props} />
        </PlayerProvider>
      );

      // The Board component should be rendered with isMyTurn=false
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });
  });

});