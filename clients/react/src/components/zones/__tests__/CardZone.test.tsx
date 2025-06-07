import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { CardZone } from '../CardZone';

describe('CardZone', () => {
  const mockOnCardClick = vi.fn();
  const mockOnZoneClick = vi.fn();

  beforeEach(() => {
    mockOnCardClick.mockClear();
    mockOnZoneClick.mockClear();
  });

  describe('Zone Types', () => {
    it('renders hand zone with cards', () => {
      const cards = [
        { entity: 'card_hearts_a' },
        { entity: 'card_spades_k' },
        { entity: 'card_diamonds_10' }
      ];

      const entityDefinitions = {
        card_hearts_a: { id: 'card_hearts_a', name: 'Ace of Hearts', props: { rank: 'A', suit: 'hearts' } },
        card_spades_k: { id: 'card_spades_k', name: 'King of Spades', props: { rank: 'K', suit: 'spades' } },
        card_diamonds_10: { id: 'card_diamonds_10', name: '10 of Diamonds', props: { rank: '10', suit: 'diamonds' } }
      };

      render(
        <CardZone 
          id="hand_p1"
          cards={cards}
          entityDefinitions={entityDefinitions}
          onCardClick={mockOnCardClick}
          metadata={{ shape: 'list' }}
        />
      );

      // Cards show symbols in text and full names in title attributes
      expect(screen.getByText('A')).toBeInTheDocument();
      expect(screen.getByText('♥')).toBeInTheDocument();
      expect(screen.getByText('K')).toBeInTheDocument();
      expect(screen.getByText('♠')).toBeInTheDocument();
      expect(screen.getByText('10')).toBeInTheDocument();
      expect(screen.getByText('♦')).toBeInTheDocument();
      
      // Check title attributes for full names
      expect(screen.getByTitle('Ace of Hearts')).toBeInTheDocument();
      expect(screen.getByTitle('King of Spades')).toBeInTheDocument();
      expect(screen.getByTitle('10 of Diamonds')).toBeInTheDocument();
    });

    it('renders deck zone as stack', () => {
      const cards = Array(52).fill({ entity: 'card_back' });
      
      render(
        <CardZone 
          id="deck"
          cards={cards}
          metadata={{ shape: 'stack' }}
          onCardClick={mockOnCardClick}
        />
      );

      // Stack should show count
      expect(screen.getByText('52')).toBeInTheDocument();
    });

    it('renders empty zone with placeholder', () => {
      render(
        <CardZone 
          id="discard"
          cards={[]}
          metadata={{ placeholder: 'Discard Pile' }}
          onCardClick={mockOnCardClick}
        />
      );

      expect(screen.getByText('Discard Pile')).toBeInTheDocument();
    });

    it('renders single card zone', () => {
      const cards = [{ entity: 'card_clubs_7' }];
      const entityDefinitions = {
        card_clubs_7: { id: 'card_clubs_7', name: '7 of Clubs' }
      };

      render(
        <CardZone 
          id="current_card"
          cards={cards}
          entityDefinitions={entityDefinitions}
          metadata={{ shape: 'single' }}
          onCardClick={mockOnCardClick}
        />
      );

      expect(screen.getByText('7 of Clubs')).toBeInTheDocument();
    });
  });

  describe('Card Display', () => {
    it('shows card faces when visible', () => {
      const cards = [{ entity: 'card_hearts_q' }];
      const entityDefinitions = {
        card_hearts_q: { 
          id: 'card_hearts_q', 
          name: 'Queen of Hearts',
          props: { rank: 'Q', suit: 'hearts', faceUp: true }
        }
      };

      render(
        <CardZone 
          id="table"
          cards={cards}
          entityDefinitions={entityDefinitions}
          onCardClick={mockOnCardClick}
        />
      );

      expect(screen.getByText('Q')).toBeInTheDocument();
      expect(screen.getByText('♥')).toBeInTheDocument();
      expect(screen.getByTitle('Queen of Hearts')).toBeInTheDocument();
    });

    it('shows card backs when face down', () => {
      const cards = [{ entity: 'card_spades_5' }];
      const entityDefinitions = {
        card_spades_5: { 
          id: 'card_spades_5', 
          props: { rank: '5', suit: 'spades', faceUp: false }
        }
      };

      render(
        <CardZone 
          id="opponent_hand"
          cards={cards}
          entityDefinitions={entityDefinitions}
          visibility="back"
          onCardClick={mockOnCardClick}
        />
      );

      // Should show card back, not the face
      expect(screen.queryByText('5♠')).not.toBeInTheDocument();
      expect(screen.getByText('🂠')).toBeInTheDocument(); // Card back unicode
    });

    it('renders cards with custom styling', () => {
      const cards = [{ entity: 'special_card' }];
      const entityDefinitions = {
        special_card: { 
          id: 'special_card', 
          name: 'Golden Card',
          props: { 
            color: '#FFD700',
            borderStyle: 'thick'
          }
        }
      };

      render(
        <CardZone 
          id="special_zone"
          cards={cards}
          entityDefinitions={entityDefinitions}
          onCardClick={mockOnCardClick}
        />
      );

      const card = screen.getByText('Golden Card');
      expect(card.parentElement).toHaveStyle({ 
        backgroundColor: '#FFD700',
        borderWidth: '3px'
      });
    });
  });

  describe('Interactions', () => {
    it('calls onCardClick with correct index', () => {
      const cards = [
        { entity: 'card1' },
        { entity: 'card2' },
        { entity: 'card3' }
      ];

      render(
        <CardZone 
          id="hand"
          cards={cards}
          onCardClick={mockOnCardClick}
        />
      );

      const cardElements = screen.getAllByRole('button');
      
      fireEvent.click(cardElements[0]);
      expect(mockOnCardClick).toHaveBeenCalledWith('hand', 0);

      fireEvent.click(cardElements[2]);
      expect(mockOnCardClick).toHaveBeenCalledWith('hand', 2);
    });

    it('calls onZoneClick when clicking empty zone', () => {
      render(
        <CardZone 
          id="draw_pile"
          cards={[]}
          onZoneClick={mockOnZoneClick}
          hasZoneAction={true}
        />
      );

      const zone = screen.getByRole('button');
      fireEvent.click(zone);
      
      expect(mockOnZoneClick).toHaveBeenCalled();
    });

    it('shows action hints on cards', () => {
      const cards = [
        { entity: 'card1' },
        { entity: 'card2' }
      ];

      const possibleActions = {
        '0': { action: 'play', direction: 'Play this card' },
        '1': { action: 'discard', direction: 'Discard this card' }
      };

      render(
        <CardZone 
          id="hand"
          cards={cards}
          possibleActions={possibleActions}
          onCardClick={mockOnCardClick}
        />
      );

      const cardElements = screen.getAllByRole('button');
      
      // Cards with actions should show hints on hover
      expect(cardElements[0]).toHaveAttribute('title', 'Play this card');
      expect(cardElements[1]).toHaveAttribute('title', 'Discard this card');
    });
  });

  describe('Visibility Rules', () => {
    it('respects player-specific visibility', () => {
      const cards = [
        { entity: 'card1' },
        { entity: 'card2' }
      ];

      // Other player's hand - should be hidden
      render(
        <CardZone 
          id="hand_p2"
          cards={cards}
          visibility="count"
          you="p1"
          onCardClick={mockOnCardClick}
        />
      );

      // Should show count only
      expect(screen.getByText('2 cards')).toBeInTheDocument();
      expect(screen.queryByText('card1')).not.toBeInTheDocument();
    });

    it('shows own cards fully', () => {
      const cards = [{ entity: 'card_diamonds_j' }];
      const entityDefinitions = {
        card_diamonds_j: { 
          id: 'card_diamonds_j', 
          name: 'Jack of Diamonds',
          props: { rank: 'J', suit: 'diamonds' }
        }
      };

      render(
        <CardZone 
          id="hand_p1"
          cards={cards}
          entityDefinitions={entityDefinitions}
          you="p1"
          onCardClick={mockOnCardClick}
        />
      );

      expect(screen.getByText('J')).toBeInTheDocument();
      expect(screen.getByText('♦')).toBeInTheDocument();
      expect(screen.getByTitle('Jack of Diamonds')).toBeInTheDocument();
    });
  });

  describe('Special Zones', () => {
    it('renders meld/set zones', () => {
      const cards = [
        { entity: 'card_hearts_7' },
        { entity: 'card_diamonds_7' },
        { entity: 'card_clubs_7' }
      ];

      const entityDefinitions = {
        card_hearts_7: { id: 'card_hearts_7', props: { rank: '7', suit: 'hearts' } },
        card_diamonds_7: { id: 'card_diamonds_7', props: { rank: '7', suit: 'diamonds' } },
        card_clubs_7: { id: 'card_clubs_7', props: { rank: '7', suit: 'clubs' } }
      };

      render(
        <CardZone 
          id="meld_1"
          cards={cards}
          entityDefinitions={entityDefinitions}
          metadata={{ 
            type: 'meld',
            label: 'Three 7s'
          }}
          onCardClick={mockOnCardClick}
        />
      );

      expect(screen.getByText('Three 7s')).toBeInTheDocument();
      expect(screen.getAllByText('7')).toHaveLength(3);
      expect(screen.getByText('♥')).toBeInTheDocument();
      expect(screen.getByText('♦')).toBeInTheDocument();
      expect(screen.getByText('♣')).toBeInTheDocument();
    });

    it('renders spread zones (like tricks)', () => {
      const cards = [
        { entity: 'card_spades_a' },
        { entity: 'card_spades_k' },
        { entity: 'card_spades_q' },
        { entity: 'card_spades_j' }
      ];

      render(
        <CardZone 
          id="trick_current"
          cards={cards}
          metadata={{ shape: 'spread' }}
          onCardClick={mockOnCardClick}
        />
      );

      // All cards should be visible in spread
      const cardElements = screen.getAllByRole('button');
      expect(cardElements).toHaveLength(4);
    });
  });
});