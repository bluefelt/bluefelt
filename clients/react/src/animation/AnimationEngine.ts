/**
 * Animation Engine for orchestrating visual animations based on game state changes.
 * 
 * Manages animation queues, timing, and coordinates between patches and visual updates.
 */

import {
  AnimationType,
  type AnimationPlan,
  type AnimationResult,
  type AnimationConfig,
  type PatchOperation,
  ANIMATION_CLASSES,
  type EntityMovementMetadata
} from './AnimationTypes';
import { PatchAnalyzer, type AnalysisContext } from './PatchAnalyzer';
import { AudioManager } from './AudioManager';

export interface AnimationEngineCallbacks {
  onAnimationStart: (animation: AnimationPlan) => void;
  onAnimationComplete: (result: AnimationResult) => void;
  onQueueEmpty: () => void;
}

export class AnimationEngine {
  private analyzer: PatchAnalyzer;
  private audioManager: AudioManager;
  private currentAnimations: Map<string, AnimationPlan> = new Map();
  private pendingQueue: AnimationPlan[] = [];
  private isProcessing = false;
  private callbacks?: AnimationEngineCallbacks;
  
  constructor(callbacks?: AnimationEngineCallbacks) {
    this.analyzer = new PatchAnalyzer();
    this.audioManager = new AudioManager();
    this.callbacks = callbacks;
    
    // Create placeholder sounds on initialization (skip in tests)
    if (!this.isTestEnvironment()) {
      this.audioManager.createPlaceholderSounds();
    }
  }
  
  private isTestEnvironment(): boolean {
    return typeof process !== 'undefined' && process.env.NODE_ENV === 'test' ||
           typeof window !== 'undefined' && (window as any).__vitest__;
  }
  
  /**
   * Process a patch and potentially create animations
   */
  async processAnimatablePatch(
    patch: PatchOperation,
    currentState: any,
    config: AnimationConfig,
    currentPlayer?: string,
    gameId?: string
  ): Promise<{ animated: boolean; newState: any; animationPlan?: AnimationPlan }> {
    
    // Skip if animations are disabled
    if (!config.enableAnimations || config.reduceMotion) {
      return { animated: false, newState: currentState };
    }
    
    // Debug logging with more detail
    console.log('[AnimationEngine] Processing patch:', {
      path: patch.path,
      op: patch.op,
      value: patch.value,
      currentPlayer
    });
    
    // Analyze patch for animation potential
    const context: AnalysisContext = {
      patch,
      currentState,
      currentPlayer,
      gameId
    };
    
    const animationPlan = this.analyzer.analyzeForAnimation(context);
    
    if (animationPlan) {
      console.log('[AnimationEngine] Animation plan created:', {
        id: animationPlan.id,
        type: animationPlan.type,
        targetPath: animationPlan.targetPath,
        metadata: animationPlan.metadata
      });
      
      // Store current and target states for animation
      animationPlan.fromState = this.captureStateSnapshot(currentState, patch.path);
      animationPlan.toState = this.predictStateAfterPatch(currentState, patch);
      
      // Queue the animation
      await this.queueAnimation(animationPlan, config);
      
      return { 
        animated: true, 
        newState: currentState, // Don't apply patch immediately
        animationPlan 
      };
    }
    
    console.log('[AnimationEngine] No animation plan created for patch path:', patch.path);
    return { animated: false, newState: currentState };
  }
  
  /**
   * Queue an animation for execution
   */
  async queueAnimation(animation: AnimationPlan, config: AnimationConfig): Promise<void> {
    // Check for duplicates
    const isDuplicate = this.pendingQueue.some(existing => 
      this.analyzer.canDeduplicateAnimation(existing, animation)
    );
    
    if (isDuplicate) {
      return;
    }
    
    // Add to queue
    this.pendingQueue.push(animation);
    
    // Sort by priority
    this.pendingQueue = this.analyzer.sortAnimationPlans(this.pendingQueue);
    
    // Start processing if not already doing so
    if (!this.isProcessing) {
      this.processQueue(config);
    }
  }
  
  /**
   * Process the animation queue
   */
  private async processQueue(config: AnimationConfig): Promise<void> {
    this.isProcessing = true;
    
    while (this.pendingQueue.length > 0) {
      const animation = this.pendingQueue.shift()!;
      
      try {
        await this.executeAnimation(animation, config);
        
        // Wait for stillness between animations
        if (this.pendingQueue.length > 0 && config.stillnessBetween > 0) {
          await this.delay(config.stillnessBetween);
        }
      } catch (error) {
        console.error('[AnimationEngine] Animation failed:', error);
        this.callbacks?.onAnimationComplete({
          success: false,
          animationId: animation.id,
          actualDuration: 0,
          error: error instanceof Error ? error.message : 'Unknown error'
        });
      }
    }
    
    this.isProcessing = false;
    this.callbacks?.onQueueEmpty();
  }
  
  /**
   * Update audio configuration
   */
  updateAudioConfig(config: AnimationConfig): void {
    this.audioManager.updateConfig({
      enableSounds: config.audioEnabled,
      volume: config.audioVolume,
      enableTurnNotifications: config.enableTurnNotifications,
      enableActionSounds: config.enableActionSounds
    });
  }
  
  /**
   * Execute a single animation
   */
  private async executeAnimation(animation: AnimationPlan, config: AnimationConfig): Promise<void> {
    const startTime = performance.now();
    
    // Defensive checks to prevent NaN durations
    const baseDuration = animation.duration || 400; // Fallback if duration is undefined
    const speed = config.speed || 1.0; // Fallback if speed is invalid
    let adjustedDuration = baseDuration / speed;
    
    // Final check for NaN
    if (isNaN(adjustedDuration) || adjustedDuration <= 0) {
      console.warn('[AnimationEngine] Invalid duration calculated, using fallback:', {
        originalDuration: animation.duration,
        speed: config.speed,
        animationType: animation.type
      });
      adjustedDuration = 400; // Safe fallback duration
    }
    
    this.currentAnimations.set(animation.id, animation);
    this.callbacks?.onAnimationStart(animation);
    
    // Play sound effect for this animation
    if (config.audioEnabled) {
      await this.audioManager.playAnimationSound(animation.type, animation.metadata);
    }
    
    try {
      switch (animation.type) {
        case AnimationType.ENTITY_MOVEMENT:
          await this.animateEntityMovement(animation, adjustedDuration);
          break;
        case AnimationType.ENTITY_SPAWN:
          await this.animateEntitySpawn(animation, adjustedDuration);
          break;
        case AnimationType.ENTITY_DESTROY:
          await this.animateEntityDestroy(animation, adjustedDuration);
          break;
        case AnimationType.ZONE_TRANSFER:
          await this.animateZoneTransfer(animation, adjustedDuration);
          break;
        case AnimationType.SELECTION_CHANGE:
          await this.animateSelectionChange(animation, adjustedDuration);
          break;
        case AnimationType.TURN_CHANGE:
          await this.animateTurnChange(animation, adjustedDuration);
          break;
        case AnimationType.PHASE_CHANGE:
          await this.animatePhaseChange(animation, adjustedDuration);
          break;
        case AnimationType.GAME_END:
          await this.animateGameEnd(animation, adjustedDuration);
          break;
        case AnimationType.CARD_DEAL:
          await this.animateCardDeal(animation, adjustedDuration);
          break;
        case AnimationType.CARD_FLIP:
          await this.animateCardFlip(animation, adjustedDuration);
          break;
        case AnimationType.CARD_SHUFFLE:
          await this.animateCardShuffle(animation, adjustedDuration);
          break;
        case AnimationType.CARD_COLLECT:
          await this.animateCardCollect(animation, adjustedDuration);
          break;
        case AnimationType.PIECE_CAPTURE:
          await this.animatePieceCapture(animation, adjustedDuration);
          break;
        case AnimationType.SCORE_CHANGE:
          await this.animateScoreChange(animation, adjustedDuration);
          break;
        case AnimationType.SHAKE_ERROR:
          await this.animateShakeError(animation, adjustedDuration);
          break;
        default:
          console.warn('[AnimationEngine] Unknown animation type:', animation.type);
      }
      
      const actualDuration = performance.now() - startTime;
      
      this.callbacks?.onAnimationComplete({
        success: true,
        animationId: animation.id,
        actualDuration
      });
      
    } finally {
      this.currentAnimations.delete(animation.id);
    }
  }
  
  /**
   * Animate entity movement between positions
   */
  private async animateEntityMovement(animation: AnimationPlan, duration: number): Promise<void> {
    const metadata = animation.metadata as EntityMovementMetadata;
    
    // Wait for React to process state updates and render
    await new Promise(resolve => setTimeout(resolve, 0)); // Allow microtasks to complete
    await new Promise(resolve => requestAnimationFrame(resolve)); // Wait for next frame
    
    // Try multiple times to find the element (React might need more time)
    let targetElement: Element | null = null;
    for (let i = 0; i < 10; i++) {
      targetElement = this.findElementByPath(animation.targetPath, true);
      if (targetElement) break;
      if (import.meta.env.DEV && i > 0) {
        console.log(`[AnimationEngine] Movement retry ${i + 1}/10 - Element not found yet for path: ${animation.targetPath}`);
      }
      await new Promise(resolve => setTimeout(resolve, 20));
    }
    
    console.log('[AnimationEngine] Entity movement animation:', {
      targetPath: animation.targetPath,
      elementFound: !!targetElement,
      isGravityDrop: metadata.isGravityDrop,
      fromPosition: metadata.fromPosition,
      toPosition: metadata.toPosition,
      metadata
    });
    
    if (targetElement && metadata.isGravityDrop && metadata.fromPosition && metadata.toPosition) {
      console.log('[AnimationEngine] Executing gravity drop animation');
      // Gravity drop animation (Connect 4 style)
      targetElement.classList.add(ANIMATION_CLASSES.MOVING);
      
      // Calculate the drop distance
      const cellHeight = targetElement.offsetHeight;
      const dropDistance = (metadata.toPosition.row - metadata.fromPosition.row) * cellHeight;
      
      // Create dropping effect with bounce
      const keyframes = [
        { 
          transform: `translateY(-${dropDistance}px) scale(0.8)`,
          opacity: '0'
        },
        {
          transform: `translateY(-${dropDistance}px) scale(1)`,
          opacity: '1',
          offset: 0.1
        },
        {
          transform: 'translateY(0) scale(1)',
          opacity: '1',
          offset: 0.8
        },
        {
          transform: 'translateY(-10px) scale(1.05)',
          opacity: '1',
          offset: 0.9
        },
        {
          transform: 'translateY(0) scale(1)',
          opacity: '1'
        }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration: Math.max(duration * 1.5, 300), // Slower for gravity effect with minimum duration
        easing: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)' // Ease-out for realistic gravity
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.MOVING);
    } else if (targetElement) {
      // Standard movement animation
      targetElement.classList.add(ANIMATION_CLASSES.MOVING);
      
      // Create movement effect
      const keyframes = [
        { transform: 'scale(1)', opacity: '1' },
        { transform: 'scale(1.1)', opacity: '0.8', offset: 0.5 },
        { transform: 'scale(1)', opacity: '1' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in-out'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.MOVING);
    } else {
      // Fallback delay if element not found
      await this.delay(duration);
    }
  }
  
  /**
   * Animate entity spawning
   */
  private async animateEntitySpawn(animation: AnimationPlan, duration: number): Promise<void> {
    // Wait a frame for React to render the new element
    await new Promise(resolve => requestAnimationFrame(resolve));
    
    // Try multiple times to find the element (React might need more time)
    let targetElement: Element | null = null;
    for (let i = 0; i < 10; i++) {
      targetElement = this.findElementByPath(animation.targetPath, true);
      if (targetElement) break;
      if (import.meta.env.DEV && i > 0) {
        console.log(`[AnimationEngine] Spawn retry ${i + 1}/10 - Element not found yet for path: ${animation.targetPath}`);
      }
      await new Promise(resolve => setTimeout(resolve, 20));
    }
    
    console.log('[AnimationEngine] Entity spawn animation:', {
      targetPath: animation.targetPath,
      elementFound: !!targetElement,
      metadata: animation.metadata
    });
    
    if (targetElement) {
      console.log('[AnimationEngine] Animating element:', targetElement);
      targetElement.classList.add(ANIMATION_CLASSES.SPAWNING);
      
      // Check if this is a Tic-Tac-Toe piece for a more subtle animation
      const gameId = (animation.metadata as any).gameId;
      const isYourPiece = (animation.metadata as any).isYourPiece;
      
      let keyframes;
      if (gameId === 'tic-tac-toe' || !(animation.metadata as any).isGravityDrop) {
        // Subtle placement animation for games like Tic-Tac-Toe
        keyframes = [
          { transform: 'scale(0)', opacity: '0' },
          { transform: 'scale(1.1)', opacity: '0.8', offset: 0.7 },
          { transform: 'scale(1)', opacity: '1' }
        ];
      } else {
        // Original spinning animation for other games
        keyframes = [
          { transform: 'scale(0) rotate(0deg)', opacity: '0' },
          { transform: 'scale(1.2) rotate(180deg)', opacity: '0.8', offset: 0.7 },
          { transform: 'scale(1) rotate(360deg)', opacity: '1' }
        ];
      }
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'cubic-bezier(0.175, 0.885, 0.32, 1.275)'
      });
      
      // Play sound effect based on piece ownership
      if (isYourPiece) {
        this.audioManager.playSound('place_yours_soft');
      } else {
        this.audioManager.playSound('place_opponent_soft'); // Slightly quieter for opponent
      }
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.SPAWNING);
    } else {
      console.log('[AnimationEngine] No element found for animation at path:', animation.targetPath);
      await this.delay(duration);
    }
  }
  
  /**
   * Animate entity destruction
   */
  private async animateEntityDestroy(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.DESTROYING);
      
      const keyframes = [
        { transform: 'scale(1)', opacity: '1' },
        { transform: 'scale(0.8)', opacity: '0.5', offset: 0.3 },
        { transform: 'scale(0)', opacity: '0' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.DESTROYING);
    } else {
      await this.delay(duration);
    }
  }
  
  /**
   * Animate zone transfers (card games)
   */
  private async animateZoneTransfer(animation: AnimationPlan, duration: number): Promise<void> {
    // For now, just use a simple delay - this would be enhanced with actual card movement
    await this.delay(duration);
  }
  
  /**
   * Animate selection changes
   */
  private async animateSelectionChange(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.SELECTING);
      
      const keyframes = [
        { borderColor: 'transparent', boxShadow: 'none' },
        { borderColor: '#3b82f6', boxShadow: '0 0 0 2px rgba(59, 130, 246, 0.5)' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration: duration * 0.5, // Faster selection animation
        easing: 'ease-out'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.SELECTING);
    } else {
      await this.delay(duration);
    }
  }
  
  /**
   * Animate turn changes
   */
  private async animateTurnChange(animation: AnimationPlan, duration: number): Promise<void> {
    // Find current player indicator and animate
    const currentPlayerElement = document.querySelector('[data-current-player="true"]');
    
    if (currentPlayerElement) {
      currentPlayerElement.classList.add(ANIMATION_CLASSES.TURN_HIGHLIGHT);
      
      const keyframes = [
        { backgroundColor: 'transparent' },
        { backgroundColor: 'rgba(34, 197, 94, 0.2)', offset: 0.5 },
        { backgroundColor: 'transparent' }
      ];
      
      const animationEffect = currentPlayerElement.animate(keyframes, {
        duration,
        easing: 'ease-in-out'
      });
      
      await animationEffect.finished;
      currentPlayerElement.classList.remove(ANIMATION_CLASSES.TURN_HIGHLIGHT);
    } else {
      await this.delay(duration);
    }
  }
  
  /**
   * Animate phase changes
   */
  private async animatePhaseChange(animation: AnimationPlan, duration: number): Promise<void> {
    // Simple phase transition for now
    await this.delay(duration);
  }
  
  /**
   * Animate game end with celebration effects
   */
  private async animateGameEnd(animation: AnimationPlan, duration: number): Promise<void> {
    const gameContainer = document.querySelector('[data-game-container]');
    const metadata = animation.metadata;
    const isVictory = metadata?.isYou === true;
    const isDefeat = metadata?.isYou === false && !metadata?.tie;
    
    if (gameContainer) {
      gameContainer.classList.add(ANIMATION_CLASSES.GAME_END_CELEBRATION);
      
      if (isVictory) {
        await this.playVictoryAnimation(gameContainer, duration);
      } else if (isDefeat) {
        await this.playDefeatAnimation(gameContainer, duration);
      } else {
        // Tie or neutral end
        await this.playNeutralEndAnimation(gameContainer, duration);
      }
      
      gameContainer.classList.remove(ANIMATION_CLASSES.GAME_END_CELEBRATION);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Victory celebration animation
   */
  private async playVictoryAnimation(container: Element, duration: number): Promise<void> {
    // Create confetti effect
    this.createConfettiEffect(container);
    
    // Main victory animation
    const victoryKeyframes = [
      { 
        transform: 'scale(1)', 
        filter: 'brightness(1) hue-rotate(0deg)',
        boxShadow: '0 0 0px rgba(34, 197, 94, 0)'
      },
      { 
        transform: 'scale(1.05)', 
        filter: 'brightness(1.2) hue-rotate(10deg)',
        boxShadow: '0 0 30px rgba(34, 197, 94, 0.8)',
        offset: 0.3
      },
      { 
        transform: 'scale(0.98)', 
        filter: 'brightness(1.1) hue-rotate(-5deg)',
        boxShadow: '0 0 20px rgba(34, 197, 94, 0.6)',
        offset: 0.6
      },
      { 
        transform: 'scale(1.02)', 
        filter: 'brightness(1.15) hue-rotate(5deg)',
        boxShadow: '0 0 25px rgba(34, 197, 94, 0.7)',
        offset: 0.8
      },
      { 
        transform: 'scale(1)', 
        filter: 'brightness(1) hue-rotate(0deg)',
        boxShadow: '0 0 0px rgba(34, 197, 94, 0)'
      }
    ];
    
    const victoryEffect = container.animate(victoryKeyframes, {
      duration: duration * 1.5,
      easing: 'cubic-bezier(0.68, -0.55, 0.265, 1.55)',
      iterations: 2
    });
    
    await victoryEffect.finished;
  }

  /**
   * Defeat animation
   */
  private async playDefeatAnimation(container: Element, duration: number): Promise<void> {
    const defeatKeyframes = [
      { 
        transform: 'scale(1)', 
        filter: 'brightness(1) saturate(1)',
        opacity: '1'
      },
      { 
        transform: 'scale(0.95)', 
        filter: 'brightness(0.7) saturate(0.5)',
        opacity: '0.8',
        offset: 0.5
      },
      { 
        transform: 'scale(1)', 
        filter: 'brightness(0.9) saturate(0.8)',
        opacity: '0.9'
      }
    ];
    
    const defeatEffect = container.animate(defeatKeyframes, {
      duration,
      easing: 'ease-out'
    });
    
    await defeatEffect.finished;
  }

  /**
   * Neutral game end animation (tie)
   */
  private async playNeutralEndAnimation(container: Element, duration: number): Promise<void> {
    const neutralKeyframes = [
      { transform: 'scale(1)', filter: 'brightness(1)' },
      { transform: 'scale(1.02)', filter: 'brightness(1.1)', offset: 0.5 },
      { transform: 'scale(1)', filter: 'brightness(1)' }
    ];
    
    const neutralEffect = container.animate(neutralKeyframes, {
      duration,
      easing: 'ease-in-out',
      iterations: 2
    });
    
    await neutralEffect.finished;
  }

  /**
   * Create confetti effect for victory
   */
  private createConfettiEffect(container: Element): void {
    // Create temporary confetti container
    const confettiContainer = document.createElement('div');
    confettiContainer.style.cssText = `
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      pointer-events: none;
      overflow: hidden;
      z-index: 1000;
    `;
    
    // Add to container temporarily
    if (container.parentElement) {
      container.parentElement.appendChild(confettiContainer);
    }
    
    // Create confetti particles
    const colors = ['#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7', '#dda0dd'];
    const particleCount = 30;
    
    for (let i = 0; i < particleCount; i++) {
      const particle = document.createElement('div');
      const color = colors[Math.floor(Math.random() * colors.length)];
      const size = Math.random() * 8 + 4;
      const startX = Math.random() * 100;
      const endX = startX + (Math.random() - 0.5) * 100;
      const duration = Math.random() * 2000 + 1000;
      
      particle.style.cssText = `
        position: absolute;
        width: ${size}px;
        height: ${size}px;
        background-color: ${color};
        border-radius: 50%;
        top: -10px;
        left: ${startX}%;
        pointer-events: none;
      `;
      
      confettiContainer.appendChild(particle);
      
      // Animate particle falling
      const fallAnimation = particle.animate([
        { 
          transform: 'translateY(-20px) rotate(0deg)', 
          opacity: '1' 
        },
        { 
          transform: `translateY(150vh) translateX(${endX - startX}%) rotate(720deg)`, 
          opacity: '0' 
        }
      ], {
        duration,
        easing: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)'
      });
      
      // Clean up particle when animation finishes
      fallAnimation.addEventListener('finish', () => {
        particle.remove();
      });
    }
    
    // Clean up confetti container after all particles finish
    setTimeout(() => {
      confettiContainer.remove();
    }, 4000);
  }
  
  /**
   * Find DOM element by game state path
   */
  private findElementByPath(path: string, findEntity: boolean = false): Element | null {
    // Convert paths like "/game/zones/board/cells/0/0" to DOM selectors
    const pathMatch = path.match(/\/game\/zones\/([^\/]+)\/cells\/(\d+)\/(\d+)/);
    
    if (pathMatch) {
      const [, zoneId, row, col] = pathMatch;
      
      if (findEntity) {
        // Find the entity display element within the cell
        return document.querySelector(
          `[data-entity-display="true"][data-zone="${zoneId}"][data-row="${row}"][data-col="${col}"]`
        );
      } else {
        // Find the cell element
        return document.querySelector(`[data-zone="${zoneId}"][data-row="${row}"][data-col="${col}"]`);
      }
    }
    
    // Fallback for other path types
    return document.querySelector(`[data-path="${path}"]`);
  }
  
  /**
   * Capture state snapshot for animation reference
   */
  private captureStateSnapshot(state: any, path: string): any {
    // Simple path-based extraction for now
    const pathParts = path.replace(/^\//, '').split('/');
    let current = state;
    
    for (const part of pathParts) {
      if (current && typeof current === 'object') {
        current = current[part];
      } else {
        break;
      }
    }
    
    return current;
  }
  
  /**
   * Predict state after patch application
   */
  private predictStateAfterPatch(state: any, patch: PatchOperation): any {
    // For now, just return the patch value
    // This could be enhanced with full JSON patch simulation
    return patch.value;
  }
  
  /**
   * Utility delay function
   */
  private delay(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
  
  /**
   * Cancel all running animations
   */
  cancelAllAnimations(): void {
    // Cancel current animations
    for (const animation of this.currentAnimations.values()) {
      const element = this.findElementByPath(animation.targetPath);
      if (element) {
        const runningAnimations = element.getAnimations();
        runningAnimations.forEach(anim => anim.cancel());
      }
    }
    
    this.currentAnimations.clear();
    this.pendingQueue = [];
    this.isProcessing = false;
  }
  
  /**
   * Animate card dealing
   */
  private async animateCardDeal(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.CARD_DEALING);
      
      const keyframes = [
        { 
          transform: 'translateY(-100px) scale(0.8) rotateX(90deg)', 
          opacity: '0' 
        },
        { 
          transform: 'translateY(-20px) scale(0.9) rotateX(45deg)', 
          opacity: '0.5',
          offset: 0.3
        },
        { 
          transform: 'translateY(0) scale(1) rotateX(0deg)', 
          opacity: '1' 
        }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'cubic-bezier(0.25, 0.46, 0.45, 0.94)'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.CARD_DEALING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate card flipping
   */
  private async animateCardFlip(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.CARD_FLIPPING);
      
      const keyframes = [
        { transform: 'rotateY(0deg)', transformOrigin: 'center' },
        { transform: 'rotateY(90deg)', transformOrigin: 'center', offset: 0.5 },
        { transform: 'rotateY(0deg)', transformOrigin: 'center' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in-out'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.CARD_FLIPPING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate card shuffling
   */
  private async animateCardShuffle(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.CARD_SHUFFLING);
      
      const keyframes = [
        { transform: 'translateX(0px) rotateZ(0deg)' },
        { transform: 'translateX(-5px) rotateZ(-2deg)', offset: 0.2 },
        { transform: 'translateX(5px) rotateZ(2deg)', offset: 0.4 },
        { transform: 'translateX(-3px) rotateZ(-1deg)', offset: 0.6 },
        { transform: 'translateX(3px) rotateZ(1deg)', offset: 0.8 },
        { transform: 'translateX(0px) rotateZ(0deg)' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in-out',
        iterations: 3
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.CARD_SHUFFLING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate card collection
   */
  private async animateCardCollect(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.CARD_COLLECTING);
      
      const keyframes = [
        { 
          transform: 'scale(1) translateY(0px)', 
          opacity: '1' 
        },
        { 
          transform: 'scale(0.8) translateY(-20px)', 
          opacity: '0.8',
          offset: 0.3 
        },
        { 
          transform: 'scale(0.6) translateY(-40px)', 
          opacity: '0.4',
          offset: 0.7 
        },
        { 
          transform: 'scale(0) translateY(-60px)', 
          opacity: '0' 
        }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.CARD_COLLECTING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate piece capture
   */
  private async animatePieceCapture(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath, true);
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.PIECE_CAPTURING);
      
      // First, shake effect for capture
      const shakeKeyframes = [
        { transform: 'translateX(0px)' },
        { transform: 'translateX(-3px)', offset: 0.25 },
        { transform: 'translateX(3px)', offset: 0.5 },
        { transform: 'translateX(-2px)', offset: 0.75 },
        { transform: 'translateX(0px)' }
      ];
      
      const shakeEffect = targetElement.animate(shakeKeyframes, {
        duration: duration * 0.3,
        easing: 'ease-in-out',
        iterations: 2
      });
      
      await shakeEffect.finished;
      
      // Then, disappear effect
      const disappearKeyframes = [
        { transform: 'scale(1)', opacity: '1' },
        { transform: 'scale(1.2)', opacity: '0.5', offset: 0.5 },
        { transform: 'scale(0)', opacity: '0' }
      ];
      
      const disappearEffect = targetElement.animate(disappearKeyframes, {
        duration: duration * 0.7,
        easing: 'ease-in'
      });
      
      await disappearEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.PIECE_CAPTURING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate score changes
   */
  private async animateScoreChange(animation: AnimationPlan, duration: number): Promise<void> {
    // Find score display element (this would need proper data attributes in the UI)
    const scoreElement = document.querySelector(`[data-player-score][data-player-id="${animation.metadata?.playerId}"]`);
    
    if (scoreElement) {
      scoreElement.classList.add(ANIMATION_CLASSES.SCORE_UPDATING);
      
      const keyframes = [
        { transform: 'scale(1)', color: 'inherit' },
        { transform: 'scale(1.2)', color: '#22c55e', offset: 0.5 },
        { transform: 'scale(1)', color: 'inherit' }
      ];
      
      const animationEffect = scoreElement.animate(keyframes, {
        duration,
        easing: 'ease-out'
      });
      
      await animationEffect.finished;
      scoreElement.classList.remove(ANIMATION_CLASSES.SCORE_UPDATING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Animate error feedback
   */
  private async animateShakeError(animation: AnimationPlan, duration: number): Promise<void> {
    const targetElement = this.findElementByPath(animation.targetPath) || 
                          document.querySelector('[data-game-container]');
    
    if (targetElement) {
      targetElement.classList.add(ANIMATION_CLASSES.ERROR_SHAKING);
      
      const keyframes = [
        { transform: 'translateX(0px)', borderColor: 'transparent' },
        { transform: 'translateX(-8px)', borderColor: '#ef4444', offset: 0.25 },
        { transform: 'translateX(8px)', borderColor: '#ef4444', offset: 0.5 },
        { transform: 'translateX(-4px)', borderColor: '#ef4444', offset: 0.75 },
        { transform: 'translateX(0px)', borderColor: 'transparent' }
      ];
      
      const animationEffect = targetElement.animate(keyframes, {
        duration,
        easing: 'ease-in-out'
      });
      
      await animationEffect.finished;
      targetElement.classList.remove(ANIMATION_CLASSES.ERROR_SHAKING);
    } else {
      await this.delay(duration);
    }
  }

  /**
   * Get current animation status
   */
  getStatus(): {
    isProcessing: boolean;
    currentCount: number;
    queuedCount: number;
  } {
    return {
      isProcessing: this.isProcessing,
      currentCount: this.currentAnimations.size,
      queuedCount: this.pendingQueue.length
    };
  }
}