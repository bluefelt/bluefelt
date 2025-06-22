/**
 * Audio Manager for game sound effects and audio cues
 */

import { AnimationType } from './AnimationTypes';

export interface AudioConfig {
  enableSounds: boolean;
  volume: number; // 0-1
  enableTurnNotifications: boolean;
  enableActionSounds: boolean;
}

export const DEFAULT_AUDIO_CONFIG: AudioConfig = {
  enableSounds: true,
  volume: 0.5,
  enableTurnNotifications: true,
  enableActionSounds: true
};

interface SoundEffect {
  url: string;
  volume?: number; // Override global volume
  pitch?: number; // Pitch adjustment
}

export class AudioManager {
  private audioContext: AudioContext | null = null;
  private sounds: Map<string, AudioBuffer> = new Map();
  private config: AudioConfig;
  
  // Sound definitions
  private soundLibrary: Record<string, SoundEffect> = {
    // Player placement sounds
    'place_yours_soft': { url: '/sounds/place_yours_soft.mp3' },
    'place_yours_hard': { url: '/sounds/place_yours_hard.mp3' },
    'drop_yours_bounce': { url: '/sounds/drop_yours_bounce.mp3' },
    
    // Opponent placement sounds
    'place_opponent_soft': { url: '/sounds/place_opponent_soft.mp3', volume: 0.7 },
    'place_opponent_hard': { url: '/sounds/place_opponent_hard.mp3', volume: 0.7 },
    'drop_opponent_bounce': { url: '/sounds/drop_opponent_bounce.mp3', volume: 0.7 },
    
    // Turn sounds
    'your_turn': { url: '/sounds/your_turn.mp3', volume: 0.5 },
    'opponent_turn': { url: '/sounds/opponent_turn.mp3', volume: 0.3 },
    
    // Action sounds
    'card_flip': { url: '/sounds/card_flip.mp3' },
    'card_shuffle': { url: '/sounds/card_shuffle.mp3' },
    'selection': { url: '/sounds/selection.mp3', volume: 0.2 },
    
    // Game state sounds
    'you_won': { url: '/sounds/you_won.mp3', volume: 0.8 },
    'you_lost': { url: '/sounds/you_lost.mp3', volume: 0.6 },
    'game_draw': { url: '/sounds/game_draw.mp3', volume: 0.7 },
    
    // Enhanced card sounds
    'card_deal': { url: '/sounds/card_deal.mp3' },
    'card_collect': { url: '/sounds/card_collect.mp3' },
    
    // Game action sounds
    'piece_capture': { url: '/sounds/piece_capture.mp3', volume: 0.7 },
    'score_change': { url: '/sounds/score_change.mp3', volume: 0.5 },
    'error_feedback': { url: '/sounds/error_feedback.mp3', volume: 0.4 },
    
    // Victory celebration sounds
    'victory_fanfare': { url: '/sounds/victory_fanfare.mp3', volume: 0.9 },
    'confetti_burst': { url: '/sounds/confetti_burst.mp3', volume: 0.6 }
  };
  
  constructor(config: AudioConfig = DEFAULT_AUDIO_CONFIG) {
    this.config = config;
    if (typeof window !== 'undefined') {
      this.initAudioContext();
    }
  }
  
  private isTestEnvironment(): boolean {
    return typeof process !== 'undefined' && process.env.NODE_ENV === 'test' ||
           typeof window !== 'undefined' && (window as any).__vitest__;
  }
  
  private initAudioContext() {
    try {
      this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
    } catch (error) {
      console.warn('[AudioManager] Web Audio API not supported:', error);
    }
  }
  
  /**
   * Update audio configuration
   */
  updateConfig(config: Partial<AudioConfig>) {
    this.config = { ...this.config, ...config };
  }
  
  /**
   * Preload sounds for better performance
   */
  async preloadSounds(soundIds?: string[]) {
    if (!this.audioContext) return;
    
    // In test environment, create mock buffers instead of fetching real files
    if (this.isTestEnvironment()) {
      const soundsToLoad = soundIds || Object.keys(this.soundLibrary);
      for (const soundId of soundsToLoad) {
        if (!this.sounds.has(soundId) && this.soundLibrary[soundId]) {
          // Create a simple mock buffer
          const mockBuffer = this.audioContext.createBuffer(1, 1024, 44100);
          this.sounds.set(soundId, mockBuffer);
        }
      }
      return;
    }
    
    const soundsToLoad = soundIds || Object.keys(this.soundLibrary);
    
    await Promise.all(
      soundsToLoad.map(async (soundId) => {
        if (this.sounds.has(soundId)) return;
        
        const sound = this.soundLibrary[soundId];
        if (!sound) return;
        
        try {
          const response = await fetch(sound.url);
          const arrayBuffer = await response.arrayBuffer();
          const audioBuffer = await this.audioContext!.decodeAudioData(arrayBuffer);
          this.sounds.set(soundId, audioBuffer);
        } catch (error) {
          console.warn(`[AudioManager] Failed to load sound: ${soundId}`, error);
        }
      })
    );
  }
  
  /**
   * Play a sound effect
   */
  async playSound(soundId: string, options?: { volume?: number; pitch?: number }) {
    if (!this.config.enableSounds || !this.audioContext) return;
    
    // Ensure audio context is running (required for user interaction)
    if (this.audioContext.state === 'suspended') {
      await this.audioContext.resume();
    }
    
    const soundDef = this.soundLibrary[soundId];
    if (!soundDef) {
      console.warn(`[AudioManager] Unknown sound: ${soundId}`);
      return;
    }
    
    // Load sound if not cached
    if (!this.sounds.has(soundId)) {
      await this.preloadSounds([soundId]);
    }
    
    const audioBuffer = this.sounds.get(soundId);
    if (!audioBuffer) return;
    
    // Create nodes
    const source = this.audioContext.createBufferSource();
    const gainNode = this.audioContext.createGain();
    
    // Configure
    source.buffer = audioBuffer;
    const volume = (options?.volume ?? soundDef.volume ?? 1) * this.config.volume;
    gainNode.gain.value = volume;
    
    // Apply pitch if specified
    if (options?.pitch || soundDef.pitch) {
      source.playbackRate.value = options?.pitch || soundDef.pitch || 1;
    }
    
    // Connect and play
    source.connect(gainNode);
    gainNode.connect(this.audioContext.destination);
    source.start();
  }
  
  /**
   * Play sound for animation type
   */
  async playAnimationSound(animationType: AnimationType, metadata?: any) {
    if (!this.config.enableActionSounds) return;
    
    switch (animationType) {
      case AnimationType.ENTITY_SPAWN:
        // Determine if this is the player's piece or opponent's
        if (metadata?.isYourPiece) {
          await this.playSound('place_yours_soft');
        } else {
          await this.playSound('place_opponent_soft');
        }
        break;
        
      case AnimationType.ENTITY_MOVEMENT:
        if (metadata?.isGravityDrop) {
          // Gravity drops (Connect 4 style)
          if (metadata?.isYourPiece) {
            await this.playSound('drop_yours_bounce');
          } else {
            await this.playSound('drop_opponent_bounce');
          }
        } else {
          // Regular movements
          if (metadata?.isYourPiece) {
            await this.playSound('place_yours_hard');
          } else {
            await this.playSound('place_opponent_hard');
          }
        }
        break;
        
      case AnimationType.ZONE_TRANSFER:
        await this.playSound('card_flip');
        break;
        
      case AnimationType.SELECTION_CHANGE:
        await this.playSound('selection');
        break;
        
      case AnimationType.TURN_CHANGE:
        if (this.config.enableTurnNotifications) {
          if (metadata?.isYourTurn) {
            await this.playSound('your_turn');
          } else {
            await this.playSound('opponent_turn');
          }
        }
        break;
        
      case AnimationType.GAME_END:
        if (metadata?.winner) {
          if (metadata.isYou) {
            // Play victory fanfare followed by confetti burst
            await this.playSound('victory_fanfare');
            setTimeout(() => this.playSound('confetti_burst'), 500);
          } else {
            await this.playSound('you_lost');
          }
        } else {
          await this.playSound('game_draw');
        }
        break;
        
      case AnimationType.CARD_DEAL:
        await this.playSound('card_deal');
        break;
        
      case AnimationType.CARD_FLIP:
        await this.playSound('card_flip');
        break;
        
      case AnimationType.CARD_SHUFFLE:
        await this.playSound('card_shuffle');
        break;
        
      case AnimationType.CARD_COLLECT:
        await this.playSound('card_collect');
        break;
        
      case AnimationType.PIECE_CAPTURE:
        await this.playSound('piece_capture');
        break;
        
      case AnimationType.SCORE_CHANGE:
        await this.playSound('score_change');
        break;
        
      case AnimationType.SHAKE_ERROR:
        await this.playSound('error_feedback');
        break;
    }
  }
  
  /**
   * Create placeholder sounds using Web Audio API oscillators
   */
  async createPlaceholderSounds() {
    if (!this.audioContext || this.isTestEnvironment()) return;
    
    // Helper to create simple tones
    const createTone = (frequency: number, duration: number, type: OscillatorType = 'sine'): AudioBuffer | null => {
      try {
        const sampleRate = this.audioContext!.sampleRate;
        const length = Math.floor(sampleRate * duration);
        const buffer = this.audioContext!.createBuffer(1, length, sampleRate);
        const data = buffer.getChannelData(0);
      
        for (let i = 0; i < length; i++) {
          const t = i / sampleRate;
          const envelope = Math.min(1, 10 * t) * Math.exp(-3 * t); // Attack and decay
          
          if (type === 'sine') {
            data[i] = Math.sin(2 * Math.PI * frequency * t) * envelope;
          } else if (type === 'square') {
            data[i] = (Math.sin(2 * Math.PI * frequency * t) > 0 ? 1 : -1) * envelope * 0.3;
          }
        }
        
        return buffer;
      } catch (error) {
        console.warn('[AudioManager] Failed to create audio buffer:', error);
        return null;
      }
    };
    
    // Create placeholder sounds with distinct tones for each situation
    // Your piece placement sounds (higher pitch, pleasant)
    const placeSoft = createTone(800, 0.1, 'sine');
    if (placeSoft) this.sounds.set('place_yours_soft', placeSoft);
    
    const placeHard = createTone(600, 0.15, 'sine');
    if (placeHard) this.sounds.set('place_yours_hard', placeHard);
    
    const dropBounce = createTone(400, 0.3, 'sine');
    if (dropBounce) this.sounds.set('drop_yours_bounce', dropBounce);
    
    // Opponent piece placement sounds (lower pitch, softer)
    const oppSoft = createTone(500, 0.1, 'sine');
    if (oppSoft) this.sounds.set('place_opponent_soft', oppSoft);
    
    const oppHard = createTone(400, 0.15, 'sine');
    if (oppHard) this.sounds.set('place_opponent_hard', oppHard);
    
    const oppBounce = createTone(300, 0.3, 'sine');
    if (oppBounce) this.sounds.set('drop_opponent_bounce', oppBounce);
    
    // Turn notification sounds
    const yourTurn = createTone(1000, 0.2, 'sine'); // High, attention-getting
    if (yourTurn) this.sounds.set('your_turn', yourTurn);
    
    const oppTurn = createTone(600, 0.15, 'sine'); // Lower, informative
    if (oppTurn) this.sounds.set('opponent_turn', oppTurn);
    
    // UI sounds
    const selectionSound = createTone(1200, 0.05, 'sine');
    if (selectionSound) this.sounds.set('selection', selectionSound);
    
    const flipSound = createTone(700, 0.1, 'square');
    if (flipSound) this.sounds.set('card_flip', flipSound);
    
    const shuffleSound = createTone(500, 0.2, 'square');
    if (shuffleSound) this.sounds.set('card_shuffle', shuffleSound);
    
    // Game end sounds
    const wonSound = createTone(800, 0.5, 'sine'); // Major chord feel
    if (wonSound) this.sounds.set('you_won', wonSound);
    
    const lostSound = createTone(300, 0.4, 'sine'); // Minor feel
    if (lostSound) this.sounds.set('you_lost', lostSound);
    
    const drawSound = createTone(500, 0.3, 'sine'); // Neutral
    if (drawSound) this.sounds.set('game_draw', drawSound);
    
    // Enhanced card sounds
    const dealSound = createTone(900, 0.12, 'sine');
    if (dealSound) this.sounds.set('card_deal', dealSound);
    
    const collectSound = createTone(600, 0.25, 'sine');
    if (collectSound) this.sounds.set('card_collect', collectSound);
    
    // Game action sounds
    const captureSound = createTone(200, 0.3, 'square');
    if (captureSound) this.sounds.set('piece_capture', captureSound);
    
    const scoreSound = createTone(1000, 0.2, 'sine');
    if (scoreSound) this.sounds.set('score_change', scoreSound);
    
    const errorSound = createTone(150, 0.15, 'square');
    if (errorSound) this.sounds.set('error_feedback', errorSound);
    
    // Victory celebration sounds
    const fanfareSound = createTone(1200, 0.8, 'sine'); // Triumphant
    if (fanfareSound) this.sounds.set('victory_fanfare', fanfareSound);
    
    const confettiBurst = createTone(1500, 0.3, 'sine'); // High sparkle
    if (confettiBurst) this.sounds.set('confetti_burst', confettiBurst);
    
    console.log('[AudioManager] Created placeholder sounds with distinct tones');
  }
}