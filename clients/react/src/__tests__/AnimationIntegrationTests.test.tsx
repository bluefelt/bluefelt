/**
 * Integration tests for the animation system with real game scenarios
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { AnimationEngine } from '../animation/AnimationEngine';
import { AnimationProvider, useAnimation } from '../context/AnimationContext';
import { 
  AnimationType, 
  DEFAULT_ANIMATION_CONFIG,
  type PatchOperation
} from '../animation/AnimationTypes';
import React, { useEffect } from 'react';

// Mock component that uses animation context
const TestComponent: React.FC<{ onMount?: () => void }> = ({ onMount }) => {
  const { isAnimating, state } = useAnimation();
  
  useEffect(() => {
    onMount?.();
  }, [onMount]);
  
  return (
    <div>
      <div data-testid="animation-status">
        {isAnimating ? 'Animating' : 'Idle'}
      </div>
      <div data-testid="queue-count">{state.queue?.length || 0}</div>
      <div data-game-container="true">
        <div data-zone="board">
          {[0, 1, 2].map(row => (
            <div key={row} data-row={row}>
              {[0, 1, 2].map(col => (
                <div 
                  key={col}
                  data-col={col}
                  data-row={row}
                  data-zone="board"
                  data-entity-display="true"
                >
                  Cell {row},{col}
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

// Mock requestAnimationFrame
global.requestAnimationFrame = vi.fn((callback) => {
  setTimeout(callback, 0);
  return 1;
});

describe('Animation System Integration', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    
    // Mock element.animate
    Element.prototype.animate = vi.fn(() => ({
      finished: Promise.resolve(),
      addEventListener: vi.fn(),
      cancel: vi.fn()
    })) as any;
    
    Element.prototype.getAnimations = vi.fn(() => []) as any;
  });

  afterEach(() => {
    vi.clearAllTimers();
  });

  describe('Real Game State Updates', () => {
    it('should animate a complete tic-tac-toe game', async () => {
      const { container } = render(
        <AnimationProvider>
          <TestComponent />
        </AnimationProvider>
      );

      // Get animation context through a ref
      let animationEngine: AnimationEngine | null = null;
      const TestWithRef: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <TestComponent />;
      };

      render(
        <AnimationProvider>
          <TestWithRef />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      // Simulate game moves
      const gamePatches: PatchOperation[] = [
        // X plays center
        { op: 'replace', path: '/game/zones/board/cells/1/1', value: { entity: 'x_p1' } },
        // O plays corner
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'o_p2' } },
        // X plays another spot
        { op: 'replace', path: '/game/zones/board/cells/0/1', value: { entity: 'x_p1' } },
        // O blocks
        { op: 'replace', path: '/game/zones/board/cells/2/1', value: { entity: 'o_p2' } },
        // X wins
        { op: 'replace', path: '/game/zones/board/cells/2/0', value: { entity: 'x_p1' } },
        // Game end
        { op: 'replace', path: '/ui/gameStatus', value: { state: 'ended', winner: 'p1' } }
      ];

      const currentState = {
        game: {
          zones: {
            board: {
              cells: Array(3).fill(null).map(() => Array(3).fill(null))
            }
          }
        }
      };

      // Process each patch
      for (let i = 0; i < gamePatches.length; i++) {
        const patch = gamePatches[i];
        const currentPlayer = i % 2 === 0 ? 'p1' : 'p2';
        
        await animationEngine!.processAnimatablePatch(
          patch,
          currentState,
          DEFAULT_ANIMATION_CONFIG,
          currentPlayer,
          'tic-tac-toe'
        );
      }

      // Wait for all animations to complete
      await waitFor(() => {
        const status = animationEngine!.getStatus();
        return !status.isProcessing && status.queuedCount === 0;
      }, { timeout: 2000 });

      // Verify all pieces were animated
      expect(Element.prototype.animate).toHaveBeenCalledTimes(6); // 5 pieces + 1 game end
    });

    it('should handle Connect 4 gravity animations correctly', async () => {
      const { container } = render(
        <AnimationProvider>
          <TestComponent />
        </AnimationProvider>
      );

      let animationEngine: AnimationEngine | null = null;
      const TestWithRef: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <TestComponent />;
      };

      render(
        <AnimationProvider>
          <TestWithRef />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      // Create Connect 4 board state
      const currentState = {
        game: {
          zones: {
            board: {
              cells: Array(6).fill(null).map(() => Array(7).fill(null))
            }
          }
        }
      };

      // Drop pieces in same column
      const connect4Patches: PatchOperation[] = [
        { op: 'replace', path: '/game/zones/board/cells/5/3', value: { entity: 'red_p1' } },
        { op: 'replace', path: '/game/zones/board/cells/4/3', value: { entity: 'yellow_p2' } },
        { op: 'replace', path: '/game/zones/board/cells/3/3', value: { entity: 'red_p1' } }
      ];

      // Update state as we go
      for (let i = 0; i < connect4Patches.length; i++) {
        const patch = connect4Patches[i];
        const currentPlayer = i % 2 === 0 ? 'p1' : 'p2';
        
        const result = await animationEngine!.processAnimatablePatch(
          patch,
          currentState,
          DEFAULT_ANIMATION_CONFIG,
          currentPlayer,
          'connect-four'
        );

        // Should detect gravity drop
        if (i > 0) {
          expect(result.animationPlan?.type).toBe(AnimationType.ENTITY_MOVEMENT);
          expect(result.animationPlan?.metadata.isGravityDrop).toBe(true);
        }

        // Update state for next iteration
        const match = patch.path.match(/cells\/(\d+)\/(\d+)$/);
        if (match) {
          const [, row, col] = match;
          currentState.game.zones.board.cells[parseInt(row)][parseInt(col)] = patch.value;
        }
      }

      await waitFor(() => {
        const status = animationEngine!.getStatus();
        return !status.isProcessing;
      });
    });
  });

  describe('Animation Context Integration', () => {
    it('should update animation status in context', async () => {
      const onMount = vi.fn();
      
      const { getByTestId } = render(
        <AnimationProvider>
          <TestComponent onMount={onMount} />
        </AnimationProvider>
      );

      // Initially idle
      expect(getByTestId('animation-status')).toHaveTextContent('Idle');
      expect(getByTestId('queue-count')).toHaveTextContent('0');

      // Get animation context
      let processPatches: any;
      const TestWithContext: React.FC = () => {
        const context = useAnimation();
        processPatches = context.processPatches;
        return null;
      };

      render(
        <AnimationProvider>
          <TestWithContext />
        </AnimationProvider>
      );

      // Process some patches
      const patches: PatchOperation[] = [
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'x_p1' } }
      ];

      await processPatches(patches, {}, 'p1');

      // Should show animating status
      await waitFor(() => {
        expect(getByTestId('animation-status')).toHaveTextContent('Animating');
      });
    });

    it('should handle animation preferences correctly', async () => {
      const TestWithPreferences: React.FC<{ enableAnimations: boolean }> = ({ enableAnimations }) => {
        const { updateConfig } = useAnimation();
        
        useEffect(() => {
          updateConfig({ enableAnimations });
        }, [enableAnimations]);

        const handleClick = async () => {
          const patch: PatchOperation = {
            op: 'replace',
            path: '/game/zones/board/cells/0/0',
            value: { entity: 'x_p1' }
          };
          await processPatches([patch], {}, 'p1');
        };

        return <button onClick={handleClick}>Animate</button>;
      };

      // Test with animations disabled
      const { rerender } = render(
        <AnimationProvider>
          <TestWithPreferences enableAnimations={false} />
        </AnimationProvider>
      );

      // Animations should be skipped
      expect(Element.prototype.animate).not.toHaveBeenCalled();

      // Re-enable animations
      rerender(
        <AnimationProvider>
          <TestWithPreferences enableAnimations={true} />
        </AnimationProvider>
      );

      // Now animations should work
      await waitFor(() => {
        expect(Element.prototype.animate).toHaveBeenCalled();
      });
    });
  });

  describe('Error Handling and Edge Cases', () => {
    it('should handle missing DOM elements gracefully', async () => {
      let animationEngine: AnimationEngine | null = null;
      
      const TestWithEngine: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <div>Test</div>;
      };

      render(
        <AnimationProvider>
          <TestWithEngine />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      // Process patch for non-existent element
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/99/99',
        value: { entity: 'x_p1' }
      };

      const result = await animationEngine!.processAnimatablePatch(
        patch,
        {},
        DEFAULT_ANIMATION_CONFIG,
        'p1'
      );

      // Should still create animation plan
      expect(result.animated).toBe(true);
      
      // Wait for animation to process
      await waitFor(() => {
        const status = animationEngine!.getStatus();
        return !status.isProcessing;
      });

      // Should complete without errors
      expect(animationEngine!.getStatus().currentCount).toBe(0);
    });

    it('should handle rapid patch updates', async () => {
      let animationEngine: AnimationEngine | null = null;
      
      const TestWithEngine: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <TestComponent />;
      };

      render(
        <AnimationProvider>
          <TestWithEngine />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      // Send many patches rapidly
      const patches: PatchOperation[] = [];
      for (let i = 0; i < 10; i++) {
        patches.push({
          op: 'replace',
          path: `/game/zones/board/cells/${i % 3}/${i % 3}`,
          value: { entity: `piece_${i}` }
        });
      }

      // Process all at once
      const promises = patches.map(patch => 
        animationEngine!.processAnimatablePatch(
          patch,
          {},
          { ...DEFAULT_ANIMATION_CONFIG, stillnessBetween: 10 },
          'p1'
        )
      );

      await Promise.all(promises);

      // Should queue animations properly
      const status = animationEngine!.getStatus();
      expect(status.queuedCount).toBeGreaterThan(0);

      // Wait for all to complete
      await waitFor(() => {
        const finalStatus = animationEngine!.getStatus();
        return !finalStatus.isProcessing && finalStatus.queuedCount === 0;
      }, { timeout: 3000 });
    });
  });

  describe('Sound Integration', () => {
    it('should play different sounds for player vs opponent pieces', async () => {
      let animationEngine: AnimationEngine | null = null;
      
      const TestWithEngine: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <TestComponent />;
      };

      render(
        <AnimationProvider>
          <TestWithEngine />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      // Spy on audio manager
      const audioManager = (animationEngine as any).audioManager;
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      // Player piece
      await animationEngine!.processAnimatablePatch(
        { op: 'replace', path: '/game/zones/board/cells/0/0', value: { entity: 'x_p1' } },
        {},
        { ...DEFAULT_ANIMATION_CONFIG, audioEnabled: true },
        'p1'
      );

      // Opponent piece
      await animationEngine!.processAnimatablePatch(
        { op: 'replace', path: '/game/zones/board/cells/1/1', value: { entity: 'o_p2' } },
        {},
        { ...DEFAULT_ANIMATION_CONFIG, audioEnabled: true },
        'p1'
      );

      await waitFor(() => {
        expect(playSoundSpy).toHaveBeenCalledTimes(2);
      });

      // Verify different volumes were used
      expect(playSoundSpy).toHaveBeenNthCalledWith(1, 'place_yours_soft');
      expect(playSoundSpy).toHaveBeenNthCalledWith(2, 'place_opponent_soft');
    });
  });

  describe('Performance', () => {
    it('should handle large batch of animations efficiently', async () => {
      let animationEngine: AnimationEngine | null = null;
      
      const TestWithEngine: React.FC = () => {
        const { engine } = useAnimation();
        useEffect(() => {
          animationEngine = engine;
        }, [engine]);
        return <TestComponent />;
      };

      render(
        <AnimationProvider>
          <TestWithEngine />
        </AnimationProvider>
      );

      await waitFor(() => expect(animationEngine).toBeTruthy());

      const startTime = performance.now();

      // Create a large batch of patches
      const patches: PatchOperation[] = [];
      for (let row = 0; row < 10; row++) {
        for (let col = 0; col < 10; col++) {
          patches.push({
            op: 'replace',
            path: `/game/zones/board/cells/${row}/${col}`,
            value: { entity: `piece_${row}_${col}` }
          });
        }
      }

      // Process all patches
      const config = { 
        ...DEFAULT_ANIMATION_CONFIG, 
        maxQueueSize: 20,
        stillnessBetween: 0 
      };

      for (const patch of patches) {
        await animationEngine!.processAnimatablePatch(patch, {}, config, 'p1');
      }

      const processingTime = performance.now() - startTime;

      // Should process quickly (under 1 second for 100 patches)
      expect(processingTime).toBeLessThan(1000);

      // Should respect queue size limit
      const status = animationEngine!.getStatus();
      expect(status.queuedCount).toBeLessThanOrEqual(config.maxQueueSize);
    });
  });
});