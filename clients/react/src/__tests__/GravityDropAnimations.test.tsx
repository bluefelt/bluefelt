import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AnimationEngine } from '../animation/AnimationEngine';
import { PatchAnalyzer } from '../animation/PatchAnalyzer';
import { AnimationType, type PatchOperation, type AnimationPlan } from '../animation/AnimationTypes';

describe('Gravity Drop Animations', () => {
  let animationEngine: AnimationEngine;
  let mockElement: HTMLElement;
  let originalQuerySelector: typeof document.querySelector;
  let findElementCalls: number;

  beforeEach(() => {
    // Clear all mocks
    vi.clearAllMocks();
    
    // Mock DOM element
    mockElement = {
      classList: {
        add: vi.fn(),
        remove: vi.fn(),
      },
      animate: vi.fn().mockReturnValue({
        finished: Promise.resolve(),
      }),
      getAnimations: vi.fn().mockReturnValue([]),
      offsetHeight: 100,
    } as any;

    // Track querySelector calls
    findElementCalls = 0;
    originalQuerySelector = document.querySelector;
    
    // Setup animation engine
    animationEngine = new AnimationEngine();
  });

  afterEach(async () => {
    // Cancel any pending animations
    if (animationEngine) {
      animationEngine.cancelAllAnimations();
    }
    
    // Wait for any async operations to complete
    await new Promise(resolve => setTimeout(resolve, 0));
    
    document.querySelector = originalQuerySelector;
    vi.restoreAllMocks();
  });

  describe('PatchAnalyzer - Server Hint Processing', () => {
    it('should convert entity_spawn with isGravityDrop to entity_movement', () => {
      const analyzer = new PatchAnalyzer();
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/5/3',
        value: { entity: 'disc_p1' },
        _animation: {
          type: 'entity_spawn',
          duration: 600,
          isGravityDrop: true,
          fromPosition: { row: 0, col: 3 },
          toPosition: { row: 5, col: 3 }
        } as any
      } as any;

      const context = {
        patch,
        currentState: {},
        currentPlayer: 'p1'
      };

      const animationPlan = analyzer.analyzeForAnimation(context);

      expect(animationPlan).toBeDefined();
      expect(animationPlan?.type).toBe(AnimationType.ENTITY_MOVEMENT);
      expect(animationPlan?.metadata).toMatchObject({
        isGravityDrop: true,
        fromPosition: { row: 0, col: 3 },
        toPosition: { row: 5, col: 3 },
        entity: 'disc_p1'
      });
    });

    it('should preserve isGravityDrop metadata through animation processing', async () => {
      const patch: PatchOperation = {
        op: 'replace',
        path: '/game/zones/board/cells/4/2',
        value: { entity: 'disc_p2' },
        _animation: {
          type: 'entity_spawn',
          duration: 600,
          isGravityDrop: true,
          fromPosition: { row: 0, col: 2 },
          toPosition: { row: 4, col: 2 }
        } as any
      } as any;

      // Mock querySelector to return element immediately
      document.querySelector = vi.fn(() => mockElement);

      const result = await animationEngine.processAnimatablePatch(
        patch,
        { game: { zones: { board: { cells: [] } } } },
        { enableAnimations: true, speed: 1.0 } as any,
        'p2',
        'connect-four'
      );

      expect(result.animated).toBe(true);
      expect(result.animationPlan?.type).toBe(AnimationType.ENTITY_MOVEMENT);
      expect(result.animationPlan?.metadata).toMatchObject({
        isGravityDrop: true,
        entity: 'disc_p2'
      });
    });
  });

  describe('AnimationEngine - Retry Logic', () => {
    it('should retry finding DOM element up to 10 times', async () => {
      let callCount = 0;
      
      // Mock querySelector to fail first 5 times, then succeed
      document.querySelector = vi.fn(() => {
        callCount++;
        findElementCalls++;
        if (callCount <= 5) {
          return null;
        }
        return mockElement;
      });

      const animationPlan: AnimationPlan = {
        id: 'test-animation',
        type: AnimationType.ENTITY_MOVEMENT,
        duration: 600,
        targetPath: '/game/zones/board/cells/5/3',
        metadata: {
          isGravityDrop: true,
          fromPosition: { row: 0, col: 3 },
          toPosition: { row: 5, col: 3 },
          entity: 'disc_p1'
        },
        priority: 1,
        fromState: null,
        toState: null
      };

      // Spy on console.log to verify retry messages
      const consoleSpy = vi.spyOn(console, 'log');
      // Set development mode for retry logging
      vi.stubGlobal('import', { meta: { env: { DEV: true } } });

      // Process the animation
      await (animationEngine as any).animateEntityMovement(animationPlan, 600);

      // Should have called querySelector multiple times
      expect(callCount).toBe(6); // Failed 5 times, succeeded on 6th
      
      // Should have logged retry messages (5 retries)
      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Movement retry 2/10')
      );

      // Animation should have been applied
      expect(mockElement.animate).toHaveBeenCalled();
    });

    it('should handle case where element is never found', async () => {
      // Create a fresh animation engine for this test
      const testEngine = new AnimationEngine();
      
      // Mock querySelector to always return null
      const mockQuerySelector = vi.fn(() => null);
      document.querySelector = mockQuerySelector;

      const animationPlan: AnimationPlan = {
        id: 'test-animation',
        type: AnimationType.ENTITY_MOVEMENT,
        duration: 600,
        targetPath: '/game/zones/board/cells/5/3',
        metadata: {
          isGravityDrop: true,
          fromPosition: { row: 0, col: 3 },
          toPosition: { row: 5, col: 3 },
          entity: 'disc_p1'
        },
        priority: 1,
        fromState: null,
        toState: null
      };

      // Process the animation directly
      await (testEngine as any).animateEntityMovement(animationPlan, 600);

      // Count calls specifically for our test path
      const callsForOurPath = mockQuerySelector.mock.calls.filter(call => {
        return call.length === 0 || (call[0] && call[0].includes && call[0].includes('5') && call[0].includes('3'));
      }).length;
      
      // Should have tried 10 times for our specific path
      expect(callsForOurPath).toBe(10);

      // Animation should not have been called
      expect(mockElement.animate).not.toHaveBeenCalled();
    });
  });

  describe('AnimationEngine - Gravity Drop Animation', () => {
    it('should apply correct keyframes for gravity drop', async () => {
      document.querySelector = vi.fn(() => mockElement);

      const animationPlan: AnimationPlan = {
        id: 'test-animation',
        type: AnimationType.ENTITY_MOVEMENT,
        duration: 600,
        targetPath: '/game/zones/board/cells/5/3',
        metadata: {
          isGravityDrop: true,
          fromPosition: { row: 0, col: 3 },
          toPosition: { row: 5, col: 3 },
          entity: 'disc_p1'
        },
        priority: 1,
        fromState: null,
        toState: null
      };

      await (animationEngine as any).animateEntityMovement(animationPlan, 600);

      // Check that animate was called with correct keyframes
      expect(mockElement.animate).toHaveBeenCalledWith(
        expect.arrayContaining([
          expect.objectContaining({
            transform: expect.stringContaining('translateY(-500px)'), // 5 rows * 100px
            opacity: '0'
          }),
          expect.objectContaining({
            transform: 'translateY(0) scale(1)',
            opacity: '1'
          })
        ]),
        expect.objectContaining({
          duration: 900, // 600 * 1.5 for gravity effect
          easing: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)'
        })
      );
    });

    it('should calculate drop distance based on row difference', async () => {
      document.querySelector = vi.fn(() => mockElement);

      const testCases = [
        { from: 0, to: 3, expectedDistance: 300 },
        { from: 0, to: 5, expectedDistance: 500 },
        { from: 1, to: 4, expectedDistance: 300 },
      ];

      for (const testCase of testCases) {
        const animationPlan: AnimationPlan = {
          id: 'test-animation',
          type: AnimationType.ENTITY_MOVEMENT,
          duration: 600,
          targetPath: `/game/zones/board/cells/${testCase.to}/2`,
          metadata: {
            isGravityDrop: true,
            fromPosition: { row: testCase.from, col: 2 },
            toPosition: { row: testCase.to, col: 2 },
            entity: 'disc_p1'
          },
          priority: 1,
          fromState: null,
          toState: null
        };

        mockElement.animate.mockClear();
        await (animationEngine as any).animateEntityMovement(animationPlan, 600);

        expect(mockElement.animate).toHaveBeenCalledWith(
          expect.arrayContaining([
            expect.objectContaining({
              transform: expect.stringContaining(`translateY(-${testCase.expectedDistance}px)`)
            })
          ]),
          expect.any(Object)
        );
      }
    });
  });

  describe('Connect Four Integration', () => {
    it('should handle rapid sequential drops without missing animations', async () => {
      const drops = [
        { path: '/game/zones/board/cells/5/0', entity: 'disc_p1' },
        { path: '/game/zones/board/cells/4/0', entity: 'disc_p2' },
        { path: '/game/zones/board/cells/3/0', entity: 'disc_p1' },
        { path: '/game/zones/board/cells/2/0', entity: 'disc_p2' },
      ];

      let animationsQueued = 0;
      let animationPlansCreated = 0;

      // Mock the PatchAnalyzer to track animation plan creation
      const analyzer = new PatchAnalyzer();
      const originalAnalyzeForAnimation = analyzer.analyzeForAnimation.bind(analyzer);
      analyzer.analyzeForAnimation = vi.fn((context) => {
        const result = originalAnalyzeForAnimation(context);
        if (result) {
          animationPlansCreated++;
        }
        return result;
      });

      // Replace the analyzer in the animation engine
      (animationEngine as any).analyzer = analyzer;

      // Mock querySelector to return element after a few tries
      let callCount = 0;
      document.querySelector = vi.fn(() => {
        callCount++;
        // Return element on every 3rd call
        if (callCount % 3 === 0) {
          return mockElement;
        }
        return null;
      });

      // Mock the queueAnimation method to track queued animations
      const originalQueueAnimation = (animationEngine as any).queueAnimation.bind(animationEngine);
      (animationEngine as any).queueAnimation = vi.fn(async (animation, config) => {
        animationsQueued++;
        return originalQueueAnimation(animation, config);
      });

      for (const drop of drops) {
        const patch: PatchOperation = {
          op: 'replace',
          path: drop.path,
          value: { entity: drop.entity },
          _animation: {
            type: 'entity_spawn',
            duration: 600,
            isGravityDrop: true,
            fromPosition: { row: 0, col: 0 },
            toPosition: { row: parseInt(drop.path.split('/')[5]), col: 0 }
          } as any
        } as any;

        await animationEngine.processAnimatablePatch(
          patch,
          { game: { zones: { board: { cells: [] } } } },
          { enableAnimations: true, speed: 1.0, reduceMotion: false } as any,
          'p1',
          'connect-four'
        );
      }

      // All animations should have been created and queued
      expect(animationPlansCreated).toBe(4);
      expect(animationsQueued).toBe(4);
    });
  });
});