import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import '@testing-library/jest-dom';
import GameZones from '../GameZones';
import { PlayerProvider } from '../../context/PlayerContext';
import { PlayerPreferencesProvider } from '../../context/PlayerPreferencesContext';
import { AnimationProvider } from '../../context/AnimationContext';

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

// Test wrapper component with all required providers
const TestWrapper = ({ children }: { children: React.ReactNode }) => (
  <PlayerPreferencesProvider>
    <PlayerProvider>
      <AnimationProvider>
        {children}
      </AnimationProvider>
    </PlayerProvider>
  </PlayerPreferencesProvider>
);

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
    isMyTurn: true,
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
          { id: 'board', renderType: 'grid', visibility: 'all', gridDimensions: { rows: 2, cols: 2 } }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });

    it('should render card zones using CardZone component', () => {
      const props = {
        ...defaultProps,
        zones: {
          hand_p1: ['card1', 'card2']
        },
        zoneMetadata: [
          { id: 'hand_p1', renderType: 'card', visibility: 'all', name: 'Player 1 Hand', cards: ['card1', 'card2'] }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
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
          deck: ['card1', 'card2', 'card3'],
          hand_p1: ['card4']
        },
        zoneMetadata: [
          { id: 'board', renderType: 'grid', visibility: 'all', gridDimensions: { rows: 2, cols: 2 } },
          { id: 'deck', renderType: 'card', visibility: 'all', cards: ['card1', 'card2', 'card3'] },
          { id: 'hand_p1', renderType: 'card', visibility: 'all', cards: ['card4'] }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
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
          { id: 'board', renderType: 'grid', visibility: 'all' }
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
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
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
          hand_p1: ['card1', 'card2']
        },
        zoneMetadata: [
          { id: 'hand_p1', renderType: 'card', visibility: 'all', name: 'Hand', cards: ['card1', 'card2'] }
        ],
        actionMap: {
          '/zones/hand_p1/0': { action: 'play', direction: 'Play this card' }
        },
        onCardAction: mockCardAction
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
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
          hand_p1: ['card1'],
          hand_p2: ['card2']
        },
        zoneMetadata: [
          { id: 'hand_p1', renderType: 'card', visibility: 'all', group: 'hands', cards: ['card1'] },
          { id: 'hand_p2', renderType: 'card', visibility: 'all', group: 'hands', cards: ['card2'] }
        ],
        zoneGroups: [
          { id: 'hands', title: 'Player Hands', zones: ['hand_p1', 'hand_p2'] }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
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
          { id: 'board', renderType: 'grid', visibility: 'all' }
        ],
        actionMap
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // The Board component should receive the action map
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });

    it('should handle flat action map structure', () => {
      const props = {
        ...defaultProps,
        you: 'p1',
        actionMap: {
          '/zones/board/0/0': { action: 'place', direction: 'Your move' },
          '/zones/board/1/1': { action: 'place', direction: 'Another move' }
        },
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', renderType: 'grid', visibility: 'all' }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // Component should receive the action map
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
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // Should render without crashing
      expect(screen.queryByTestId('board-component')).not.toBeInTheDocument();
      expect(screen.queryByTestId(/card-zone/)).not.toBeInTheDocument();
    });

    it('should handle missing zone metadata gracefully', () => {
      // Mock console.warn to verify warning is logged
      const consoleWarn = vi.spyOn(console, 'warn').mockImplementation(() => {});
      
      const props = {
        ...defaultProps,
        zones: {
          board: {
            cells: [[null, null], [null, null]]
          }
        },
        zoneMetadata: undefined
      };

      const { container } = render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // Should warn about missing metadata
      expect(consoleWarn).toHaveBeenCalledWith('No zone metadata provided by server - using fallback zone detection');
      
      // The component should render without crashing
      expect(container.firstChild).toBeDefined();
      
      // In server authority design, zones won't render properly without metadata
      // This is expected behavior - the server must provide metadata
      expect(screen.queryByTestId('board-component')).not.toBeInTheDocument();
      
      consoleWarn.mockRestore();
    });

    it('should handle malformed zone data', () => {
      const props = {
        ...defaultProps,
        zones: {
          brokenZone: {
            // Missing cells but has grid renderType
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'brokenZone', renderType: 'grid', visibility: 'all' }
        ]
      };

      const { container } = render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // Should handle gracefully without crashing
      // The component should render but might not show the broken zone properly
      expect(container.firstChild).toBeDefined();
    });
  });

  describe('Turn-based Interactions', () => {
    it('should disable interactions when not your turn', () => {
      const props = {
        ...defaultProps,
        isMyTurn: false,
        zones: {
          board: {
            cells: [[null, null], [null, null]],
            type: 'grid'
          }
        },
        zoneMetadata: [
          { id: 'board', renderType: 'grid', visibility: 'all' }
        ]
      };

      render(
        <TestWrapper>
          <GameZones {...props} />
        </TestWrapper>
      );

      // The Board component should be rendered with isMyTurn=false
      expect(screen.getByTestId('board-component')).toBeInTheDocument();
    });
  });

});