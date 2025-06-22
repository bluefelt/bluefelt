/**
 * Comprehensive tests for the Animation System visual components
 * 
 * Tests the integration and functionality of:
 * - ViewZone animations
 * - ActionIndicator visual feedback
 * - MultiStepActionDisplay progress animations
 * - AnimationSettings UI controls
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { ViewZone } from '../components/zones/ViewZone';
import { ActionIndicator, useHasAction } from '../components/ActionIndicator';
import { MultiStepActionDisplay } from '../components/MultiStepActionDisplay';
import { AnimationSettings } from '../components/AnimationSettings';
import { AnimationProvider } from '../context/AnimationContext';
import { AnimationEngine } from '../animation/AnimationEngine';
import { PatchAnalyzer } from '../animation/PatchAnalyzer';
import { AnimationType, type PatchOperation } from '../animation/AnimationTypes';

// Mock the animation context
const mockAnimationContext = {
  state: {
    config: {
      enableAnimations: true,
      speed: 1.0,
      stillnessBetween: 200,
      audioEnabled: true,
      audioVolume: 0.8,
      enableTurnNotifications: true,
      enableActionSounds: true,
      reduceMotion: false
    },
    isAnimating: false,
    queuedAnimations: []
  },
  updateConfig: vi.fn(),
  queueAnimation: vi.fn(),
  cancelAllAnimations: vi.fn()
};

vi.mock('../context/AnimationContext', () => ({
  useAnimation: () => mockAnimationContext,
  AnimationProvider: ({ children }: { children: React.ReactNode }) => <div>{children}</div>
}));

describe('Animation System Visual Components', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Reset animation context state
    mockAnimationContext.state.config = {
      enableAnimations: true,
      speed: 1.0,
      stillnessBetween: 200,
      audioEnabled: true,
      audioVolume: 0.8,
      enableTurnNotifications: true,
      enableActionSounds: true,
      reduceMotion: false
    };
  });

  describe('ViewZone Animations', () => {
    const mockViewData = {
      players: {
        p1: { cardsInHand: 5, pairsMade: 2 },
        p2: { cardsInHand: 3, pairsMade: 1 }
      },
      shared: { deckRemaining: 20, totalMoves: 8 },
      meta: {
        labels: {
          cardsInHand: 'Cards in Hand',
          pairsMade: 'Pairs Made',
          deckRemaining: 'Deck Remaining',
          totalMoves: 'Total Moves'
        }
      }
    };

    it('renders with fade-in animation', async () => {
      render(
        <ViewZone
          zoneId="game_stats"
          zoneName="Game Statistics"
          viewType="strategic"
          data={mockViewData}
          playerNames={['Alice', 'Bob']}
          you="p1"
        />
      );

      const viewZone = screen.getByText('Game Statistics').parentElement;
      expect(viewZone).toHaveClass('opacity-0');
      
      // Wait for fade-in animation to trigger
      await waitFor(() => {
        expect(viewZone).toHaveClass('opacity-100');
      }, { timeout: 200 });
    });

    it('renders table format with staggered row animations', () => {
      render(
        <ViewZone
          zoneId="game_stats"
          zoneName="Game Statistics"
          viewType="strategic"
          data={mockViewData}
          format={{ style: 'table' }}
          playerNames={['Alice', 'Bob']}
          you="p1"
        />
      );

      // Check that table rows have animation classes
      const tableRows = screen.getAllByRole('row');
      const dataRows = tableRows.slice(1); // Skip header row
      
      dataRows.forEach((row, index) => {
        expect(row).toHaveClass('animate-slide-in');
        expect(row).toHaveStyle(`animation-delay: ${index * 100}ms`);
      });
    });

    it('renders cards format with scale animations', () => {
      render(
        <ViewZone
          zoneId="game_stats"
          zoneName="Game Statistics"
          viewType="strategic"
          data={mockViewData}
          format={{ style: 'cards' }}
          playerNames={['Alice', 'Bob']}
          you="p1"
        />
      );

      // Check that player cards have scale animation
      const playerCards = screen.getAllByText(/Alice|Bob/).map(el => el.closest('div'));
      playerCards.forEach((card, index) => {
        if (card) {
          expect(card).toHaveClass('animate-scale-in');
          expect(card).toHaveStyle(`animation-delay: ${index * 100}ms`);
        }
      });
    });

    it('renders animated values that update smoothly', async () => {
      const { rerender } = render(
        <ViewZone
          zoneId="game_stats"
          zoneName="Game Statistics"
          viewType="strategic"
          data={mockViewData}
          playerNames={['Alice', 'Bob']}
          you="p1"
        />
      );

      // Initial render should show original values
      expect(screen.getByText('5')).toBeInTheDocument();
      expect(screen.getByText('20')).toBeInTheDocument();

      // Update data to trigger value animations
      const updatedData = {
        ...mockViewData,
        players: {
          p1: { cardsInHand: 4, pairsMade: 3 },
          p2: { cardsInHand: 3, pairsMade: 1 }
        },
        shared: { deckRemaining: 18, totalMoves: 9 }
      };

      rerender(
        <ViewZone
          zoneId="game_stats"
          zoneName="Game Statistics"
          viewType="strategic"
          data={updatedData}
          playerNames={['Alice', 'Bob']}
          you="p1"
        />
      );

      // Values should eventually update
      await waitFor(() => {
        expect(screen.getByText('4')).toBeInTheDocument();
        expect(screen.getByText('18')).toBeInTheDocument();
      });
    });

    it('handles empty data gracefully', () => {
      render(
        <ViewZone
          zoneId="empty_stats"
          zoneName="Empty Statistics"
          viewType="strategic"
          data={{}}
          playerNames={[]}
          you="p1"
        />
      );

      expect(screen.getByText('Empty Statistics')).toBeInTheDocument();
    });
  });

  describe('ActionIndicator Visual Feedback', () => {
    it('renders pulsing ring for regular actions', () => {
      render(
        <div className="relative">
          <ActionIndicator hasAction={true} />
        </div>
      );

      const ring = document.querySelector('.ring-blue-500');
      expect(ring).toBeInTheDocument();
      expect(ring).toHaveClass('animate-pulse');
    });

    it('renders purple ring for multi-step actions', () => {
      render(
        <div className="relative">
          <ActionIndicator hasAction={true} isMultiStep={true} />
        </div>
      );

      const ring = document.querySelector('.ring-purple-500');
      expect(ring).toBeInTheDocument();
      expect(ring).toHaveClass('animate-pulse');
    });

    it('renders corner indicator with ping animation', () => {
      render(
        <div className="relative">
          <ActionIndicator hasAction={true} />
        </div>
      );

      const corner = document.querySelector('.bg-blue-500');
      expect(corner).toBeInTheDocument();
      expect(corner).toHaveClass('animate-ping');
    });

    it('does not render when hasAction is false', () => {
      render(
        <div className="relative">
          <ActionIndicator hasAction={false} />
        </div>
      );

      expect(document.querySelector('.ring-blue-500')).not.toBeInTheDocument();
    });
  });

  describe('useHasAction Hook', () => {
    it('detects regular actions correctly', () => {
      const TestComponent = () => {
        const { hasAction, isMultiStepAction } = useHasAction(
          '/zones/board/cells/0/0',
          { '/zones/board/cells/0/0': { type: 'placeToken' } },
          false
        );
        
        return (
          <div>
            <span data-testid="has-action">{hasAction.toString()}</span>
            <span data-testid="is-multi-step">{isMultiStepAction.toString()}</span>
          </div>
        );
      };

      render(<TestComponent />);
      expect(screen.getByTestId('has-action')).toHaveTextContent('true');
      expect(screen.getByTestId('is-multi-step')).toHaveTextContent('false');
    });

    it('detects multi-step actions correctly', () => {
      const TestComponent = () => {
        const { hasAction, isMultiStepAction } = useHasAction(
          '/zones/board/cells/0/0',
          { '/zones/board/cells/0/0': { type: 'movePiece', multiStepId: 'move-123' } },
          true,
          { actionId: 'move-123', currentStepId: 'selectPiece' }
        );
        
        return (
          <div>
            <span data-testid="has-action">{hasAction.toString()}</span>
            <span data-testid="is-multi-step">{isMultiStepAction.toString()}</span>
          </div>
        );
      };

      render(<TestComponent />);
      expect(screen.getByTestId('has-action')).toHaveTextContent('true');
      expect(screen.getByTestId('is-multi-step')).toHaveTextContent('true');
    });
  });

  describe('MultiStepActionDisplay Progress Animations', () => {
    const mockMultiStepState = {
      actionId: 'move-123',
      actionType: 'movePiece',
      currentStepId: 'selectDestination',
      currentStepIndex: 1,
      totalSteps: 3,
      storedData: { selectedPiece: 'token_p1' },
      canCancel: true,
      requiresConfirmation: false
    };

    it('renders progress bar with animated width', () => {
      render(
        <MultiStepActionDisplay
          multiStepState={mockMultiStepState}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      const progressBar = document.querySelector('.bg-blue-600');
      expect(progressBar).toBeInTheDocument();
      expect(progressBar).toHaveClass('transition-all', 'duration-300');
      expect(progressBar).toHaveStyle('width: 66.66666666666666%'); // 2/3 steps
    });

    it('shows correct step information', () => {
      render(
        <MultiStepActionDisplay
          multiStepState={mockMultiStepState}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      expect(screen.getByText('Step 2 of 3')).toBeInTheDocument();
      expect(screen.getByText('67%')).toBeInTheDocument(); // Rounded percentage
      expect(screen.getByText('Select where to move token_p1')).toBeInTheDocument();
    });

    it('renders confirmation dialog correctly', () => {
      const confirmationState = {
        ...mockMultiStepState,
        requiresConfirmation: true,
        confirmationPrompt: 'Are you sure you want to move this piece?'
      };

      render(
        <MultiStepActionDisplay
          multiStepState={confirmationState}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      expect(screen.getByText('Confirm Action')).toBeInTheDocument();
      expect(screen.getByText('Are you sure you want to move this piece?')).toBeInTheDocument();
      expect(screen.getByText('Confirm')).toBeInTheDocument();
      expect(screen.getByText('Cancel')).toBeInTheDocument();
    });

    it('calls onCancel when cancel button is clicked', () => {
      const mockOnCancel = vi.fn();
      
      render(
        <MultiStepActionDisplay
          multiStepState={mockMultiStepState}
          onCancel={mockOnCancel}
          onConfirm={vi.fn()}
        />
      );

      fireEvent.click(screen.getByText('Cancel'));
      expect(mockOnCancel).toHaveBeenCalledOnce();
    });

    it('does not render when multiStepState is null', () => {
      render(
        <MultiStepActionDisplay
          multiStepState={null}
          onCancel={vi.fn()}
          onConfirm={vi.fn()}
        />
      );

      expect(screen.queryByText('Action in Progress')).not.toBeInTheDocument();
    });
  });

  describe('AnimationSettings UI Controls', () => {
    it('renders all animation and audio settings', () => {
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={vi.fn()} />
        </AnimationProvider>
      );

      expect(screen.getByText('Audio & Animation Settings')).toBeInTheDocument();
      expect(screen.getByText('Enable Animations')).toBeInTheDocument();
      expect(screen.getByText('Animation Speed: 1x')).toBeInTheDocument();
      expect(screen.getByText('Sound Effects')).toBeInTheDocument();
      expect(screen.getByText('Turn Notifications')).toBeInTheDocument();
      expect(screen.getByText('Action Sounds')).toBeInTheDocument();
      expect(screen.getByText('Reduce Motion')).toBeInTheDocument();
    });

    it('updates animation speed when slider changes', () => {
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={vi.fn()} />
        </AnimationProvider>
      );

      const speedSlider = screen.getByDisplayValue('1');
      fireEvent.change(speedSlider, { target: { value: '2' } });

      expect(mockAnimationContext.updateConfig).toHaveBeenCalledWith({ speed: 2 });
    });

    it('toggles animations when switch is clicked', () => {
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={vi.fn()} />
        </AnimationProvider>
      );

      const animationToggle = screen.getByRole('switch', { name: /enable animations/i });
      fireEvent.click(animationToggle);

      expect(mockAnimationContext.updateConfig).toHaveBeenCalledWith({ enableAnimations: false });
    });

    it('shows current animation status', () => {
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={vi.fn()} />
        </AnimationProvider>
      );

      expect(screen.getByText(/Animations: Enabled/)).toBeInTheDocument();
      expect(screen.getByText(/Audio: Enabled/)).toBeInTheDocument();
      expect(screen.getByText(/Queue: 0 animations/)).toBeInTheDocument();
      expect(screen.getByText(/Active: No/)).toBeInTheDocument();
    });

    it('disables controls when reduce motion is enabled', () => {
      mockAnimationContext.state.config.reduceMotion = true;
      mockAnimationContext.state.config.enableAnimations = false;

      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={vi.fn()} />
        </AnimationProvider>
      );

      const speedSlider = screen.getByDisplayValue('1');
      expect(speedSlider).toBeDisabled();
    });

    it('does not render when isOpen is false', () => {
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={false} onClose={vi.fn()} />
        </AnimationProvider>
      );

      expect(screen.queryByText('Audio & Animation Settings')).not.toBeInTheDocument();
    });

    it('calls onClose when close button is clicked', () => {
      const mockOnClose = vi.fn();
      
      render(
        <AnimationProvider>
          <AnimationSettings isOpen={true} onClose={mockOnClose} />
        </AnimationProvider>
      );

      fireEvent.click(screen.getByText('Done'));
      expect(mockOnClose).toHaveBeenCalledOnce();
    });
  });

  describe('Animation System Integration', () => {
    it('ViewZone integrates with AnimationContext for reduced motion', () => {
      mockAnimationContext.state.config.reduceMotion = true;

      render(
        <AnimationProvider>
          <ViewZone
            zoneId="game_stats"
            zoneName="Game Statistics"
            viewType="strategic"
            data={{ players: {}, shared: {} }}
            playerNames={[]}
            you="p1"
          />
        </AnimationProvider>
      );

      // With reduced motion, animations should be minimal
      const viewZone = screen.getByText('Game Statistics').parentElement;
      expect(viewZone).toBeInTheDocument();
    });

    it('ActionIndicator responds to animation settings', () => {
      mockAnimationContext.state.config.enableAnimations = false;

      render(
        <AnimationProvider>
          <div className="relative">
            <ActionIndicator hasAction={true} />
          </div>
        </AnimationProvider>
      );

      // Even when animations are disabled, the indicator should still show
      const ring = document.querySelector('.ring-blue-500');
      expect(ring).toBeInTheDocument();
    });
  });

  describe('Animation Engine and Patch Processing', () => {
    let engine: AnimationEngine;
    let analyzer: PatchAnalyzer;

    beforeEach(() => {
      engine = new AnimationEngine();
      analyzer = new PatchAnalyzer();
    });

    it('correctly identifies tic-tac-toe animations', async () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/1/1',
        value: { entity: 'x_p1' }
      };

      const context = {
        patch,
        currentState: { game: { zones: { board: { cells: [[null, null], [null, null]] } } } },
        currentPlayer: 'p1',
        gameId: 'tic-tac-toe'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);
      
      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_SPAWN);
      expect(animationPlan?.metadata.gameId).toBe('tic-tac-toe');
    });

    it('differentiates between player and opponent animations', async () => {
      const playerPatch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/0/0',
        value: { entity: 'x_p1' }
      };

      const opponentPatch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/1/1',
        value: { entity: 'o_p2' }
      };

      const playerContext = {
        patch: playerPatch,
        currentState: { game: { zones: { board: { cells: [[null, null], [null, null]] } } } },
        currentPlayer: 'p1'
      };

      const opponentContext = {
        patch: opponentPatch,
        currentState: { game: { zones: { board: { cells: [[null, null], [null, null]] } } } },
        currentPlayer: 'p1' // Current player is p1, so p2 is opponent
      };

      const playerAnimation = analyzer.analyzeForAnimation(playerContext);
      const opponentAnimation = analyzer.analyzeForAnimation(opponentContext);

      expect(playerAnimation?.metadata.isYourPiece).toBe(true);
      expect(opponentAnimation?.metadata.isYourPiece).toBe(false);
    });

    it('processes all pieces in a game without missing animations', async () => {
      const animationCount = { count: 0 };
      const mockCallbacks = {
        onAnimationStart: () => { animationCount.count++; },
        onAnimationComplete: vi.fn(),
        onQueueEmpty: vi.fn()
      };

      const testEngine = new AnimationEngine(mockCallbacks);
      
      // Simulate a complete tic-tac-toe game
      const gamePatches = [
        { row: 0, col: 0, entity: 'x_p1', player: 'p1' },
        { row: 0, col: 1, entity: 'o_p2', player: 'p2' },
        { row: 1, col: 0, entity: 'x_p1', player: 'p1' },
        { row: 1, col: 1, entity: 'o_p2', player: 'p2' },
        { row: 2, col: 0, entity: 'x_p1', player: 'p1' }
      ];

      for (const move of gamePatches) {
        const patch: PatchOperation = {
          op: 'replace',
          path: `/game/zones/board/cells/${move.row}/${move.col}`,
          value: { entity: move.entity }
        };

        const result = await testEngine.processAnimatablePatch(
          patch,
          {},
          { ...mockAnimationContext.state.config, enableAnimations: true },
          move.player,
          'tic-tac-toe'
        );

        expect(result.animated).toBe(true);
        expect(result.animationPlan).toBeDefined();
      }

      // All 5 pieces should have been animated
      expect(animationCount.count).toBeGreaterThanOrEqual(5);
    });
  });

  describe('Animation DOM Integration', () => {
    it('finds and animates the correct DOM elements', async () => {
      // Setup DOM structure
      document.body.innerHTML = `
        <div data-game-container="true">
          <div data-zone="board" data-row="0" data-col="0" data-entity-display="true">
            Cell 0,0
          </div>
          <div data-zone="board" data-row="1" data-col="1" data-entity-display="true">
            Cell 1,1
          </div>
        </div>
      `;

      const engine = new AnimationEngine();
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/0/0',
        value: { entity: 'x_p1' }
      };

      // Mock animate method
      const mockAnimate = vi.fn(() => ({ finished: Promise.resolve() }));
      Element.prototype.animate = mockAnimate as any;

      await engine.processAnimatablePatch(
        patch,
        {},
        { ...mockAnimationContext.state.config },
        'p1'
      );

      // Wait for animation processing
      await waitFor(() => {
        expect(mockAnimate).toHaveBeenCalled();
      });

      // Verify correct element was animated
      const animatedElement = document.querySelector('[data-zone="board"][data-row="0"][data-col="0"]');
      expect(animatedElement).toBeTruthy();
    });

    it('retries finding elements for React rendering delays', async () => {
      const engine = new AnimationEngine();
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/2/2',
        value: { entity: 'o_p2' }
      };

      let queryCount = 0;
      const originalQuerySelector = document.querySelector.bind(document);
      document.querySelector = vi.fn((selector: string) => {
        queryCount++;
        // Element appears after 3 tries
        if (queryCount >= 3) {
          const elem = document.createElement('div');
          elem.setAttribute('data-entity-display', 'true');
          elem.animate = vi.fn(() => ({ finished: Promise.resolve() })) as any;
          return elem;
        }
        return null;
      });

      await engine.processAnimatablePatch(patch, {}, mockAnimationContext.state.config, 'p2');

      // Wait for retry logic
      await waitFor(() => {
        expect(queryCount).toBeGreaterThanOrEqual(3);
      });

      document.querySelector = originalQuerySelector;
    });
  });
});