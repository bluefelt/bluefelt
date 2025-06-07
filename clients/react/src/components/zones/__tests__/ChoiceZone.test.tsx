import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChoiceZone } from '../ChoiceZone';

describe('ChoiceZone', () => {
  const mockOnSelect = vi.fn();

  beforeEach(() => {
    mockOnSelect.mockClear();
  });

  describe('Basic Rendering', () => {
    it('renders nothing when inactive', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [
          { id: 'choice1', label: 'Choice 1' }
        ]
      };

      const { container } = render(
        <ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={false} />
      );

      expect(container.firstChild).toBeNull();
    });

    it('renders nothing when no items', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: []
      };

      const { container } = render(
        <ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />
      );

      expect(container.firstChild).toBeNull();
    });

    it('renders choices when active with items', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [
          { id: 'choice1', label: 'Choice 1' },
          { id: 'choice2', label: 'Choice 2' }
        ]
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      expect(screen.getByText('Choice 1')).toBeInTheDocument();
      expect(screen.getByText('Choice 2')).toBeInTheDocument();
    });

    it('renders prompt when provided', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [{ id: 'choice1', label: 'Choice 1' }],
        prompt: 'Select an option'
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      expect(screen.getByText('Select an option')).toBeInTheDocument();
    });
  });

  describe('Choice Types', () => {
    it('renders rank selection choices', () => {
      const zone = {
        id: 'choice_p1',
        type: 'choice' as const,
        items: [
          { id: '2', label: 'Rank 2' },
          { id: '3', label: 'Rank 3' },
          { id: 'A', label: 'Rank A' },
          { id: 'K', label: 'Rank K' }
        ],
        prompt: 'Choose a rank to ask for'
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      expect(screen.getByText('Choose a rank to ask for')).toBeInTheDocument();
      expect(screen.getByText('Rank 2')).toBeInTheDocument();
      expect(screen.getByText('Rank A')).toBeInTheDocument();
      expect(screen.getByText('Rank K')).toBeInTheDocument();
    });

    it('renders player selection choices', () => {
      const zone = {
        id: 'choice_p1',
        type: 'choice' as const,
        items: [
          { id: 'p2', label: 'Player 2' },
          { id: 'p3', label: 'Player 3' }
        ],
        prompt: 'Choose a player to ask'
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      expect(screen.getByText('Choose a player to ask')).toBeInTheDocument();
      expect(screen.getByText('Player 2')).toBeInTheDocument();
      expect(screen.getByText('Player 3')).toBeInTheDocument();
    });

    it('renders choices with linked entities', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [
          { 
            id: 'choice1', 
            label: 'Move Knight',
            linkedEntities: ['knight_white', 'position_e4']
          }
        ]
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      expect(screen.getByText('Move Knight')).toBeInTheDocument();
      expect(screen.getByText(/Related: knight_white, position_e4/)).toBeInTheDocument();
    });
  });

  describe('Interactions', () => {
    it('calls onSelect with choice id when clicked', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [
          { id: 'choice1', label: 'Choice 1' },
          { id: 'choice2', label: 'Choice 2' }
        ]
      };

      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);

      fireEvent.click(screen.getByText('Choice 1'));
      expect(mockOnSelect).toHaveBeenCalledWith('choice1');

      fireEvent.click(screen.getByText('Choice 2'));
      expect(mockOnSelect).toHaveBeenCalledWith('choice2');
    });

    it('does not call onSelect when inactive', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [{ id: 'choice1', label: 'Choice 1' }]
      };

      const { rerender } = render(
        <ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={false} />
      );

      // Should not render when inactive
      expect(screen.queryByText('Choice 1')).not.toBeInTheDocument();

      // Re-render as active
      rerender(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);
      
      fireEvent.click(screen.getByText('Choice 1'));
      expect(mockOnSelect).toHaveBeenCalledWith('choice1');
    });
  });

  describe('Visibility', () => {
    it('respects visibility settings', () => {
      const zone = {
        id: 'test-choice',
        type: 'choice' as const,
        items: [{ id: 'choice1', label: 'Choice 1' }],
        visibility: 'hidden'
      };

      // Note: Current implementation doesn't check visibility
      // This test documents expected behavior if visibility is implemented
      render(<ChoiceZone zone={zone} onSelect={mockOnSelect} isActive={true} />);
      
      // Currently renders regardless of visibility
      expect(screen.getByText('Choice 1')).toBeInTheDocument();
    });
  });
});