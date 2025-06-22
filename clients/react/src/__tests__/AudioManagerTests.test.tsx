/**
 * Tests for the Audio Manager
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { AudioManager, DEFAULT_AUDIO_CONFIG } from '../animation/AudioManager';
import { AnimationType } from '../animation/AnimationTypes';

// Mock Web Audio API
class MockAudioContext {
  sampleRate = 44100;
  state = 'running' as AudioContextState;
  destination = {};
  
  createBuffer = vi.fn((channels: number, length: number, sampleRate: number) => ({
    getChannelData: vi.fn(() => new Float32Array(length))
  }));
  
  createBufferSource = vi.fn(() => ({
    buffer: null,
    connect: vi.fn(),
    start: vi.fn(),
    playbackRate: { value: 1 }
  }));
  
  createGain = vi.fn(() => ({
    gain: { value: 1 },
    connect: vi.fn()
  }));
  
  decodeAudioData = vi.fn(async () => ({
    duration: 1,
    length: 44100,
    numberOfChannels: 2,
    sampleRate: 44100
  }));
  
  resume = vi.fn(async () => {});
}

// Mock fetch
global.fetch = vi.fn(async () => ({
  arrayBuffer: async () => new ArrayBuffer(1024)
})) as any;

describe('AudioManager', () => {
  let audioManager: AudioManager;
  let mockAudioContext: MockAudioContext;

  beforeEach(() => {
    mockAudioContext = new MockAudioContext();
    global.AudioContext = MockAudioContext as any;
    audioManager = new AudioManager();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('Sound Playback', () => {
    it('should play sounds when enabled', async () => {
      const source = { 
        buffer: null, 
        connect: vi.fn(), 
        start: vi.fn(),
        playbackRate: { value: 1 }
      };
      const gainNode = { 
        gain: { value: 1 }, 
        connect: vi.fn() 
      };
      
      mockAudioContext.createBufferSource.mockReturnValue(source);
      mockAudioContext.createGain.mockReturnValue(gainNode);

      await audioManager.playSound('place_yours_soft');

      expect(source.connect).toHaveBeenCalledWith(gainNode);
      expect(gainNode.connect).toHaveBeenCalledWith(mockAudioContext.destination);
      expect(source.start).toHaveBeenCalled();
    });

    it('should not play sounds when disabled', async () => {
      audioManager.updateConfig({ enableSounds: false });

      await audioManager.playSound('place_yours_soft');

      expect(mockAudioContext.createBufferSource).not.toHaveBeenCalled();
    });

    it('should adjust volume based on config', async () => {
      const gainNode = { 
        gain: { value: 1 }, 
        connect: vi.fn() 
      };
      
      mockAudioContext.createGain.mockReturnValue(gainNode);
      audioManager.updateConfig({ volume: 0.3 });

      await audioManager.playSound('place_yours_soft');

      expect(gainNode.gain.value).toBe(0.3);
    });

    it('should apply sound-specific volume overrides', async () => {
      const gainNode = { 
        gain: { value: 1 }, 
        connect: vi.fn() 
      };
      
      mockAudioContext.createGain.mockReturnValue(gainNode);
      audioManager.updateConfig({ volume: 0.5 });

      // Place opponent sound has 0.7 volume override
      await audioManager.playSound('place_opponent_soft');

      expect(gainNode.gain.value).toBe(0.35); // 0.5 * 0.7
    });

    it('should resume suspended audio context', async () => {
      mockAudioContext.state = 'suspended';

      await audioManager.playSound('place_yours_soft');

      expect(mockAudioContext.resume).toHaveBeenCalled();
    });
  });

  describe('Animation Sound Effects', () => {
    it('should play correct sounds for entity spawn', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      // Your piece
      await audioManager.playAnimationSound(AnimationType.ENTITY_SPAWN, { isYourPiece: true });
      expect(playSoundSpy).toHaveBeenCalledWith('place_yours_soft');

      // Opponent piece
      await audioManager.playAnimationSound(AnimationType.ENTITY_SPAWN, { isYourPiece: false });
      expect(playSoundSpy).toHaveBeenCalledWith('place_opponent_soft');
    });

    it('should play gravity drop sounds for entity movement', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      // Your gravity drop
      await audioManager.playAnimationSound(AnimationType.ENTITY_MOVEMENT, { 
        isGravityDrop: true, 
        isYourPiece: true 
      });
      expect(playSoundSpy).toHaveBeenCalledWith('drop_yours_bounce');

      // Opponent gravity drop
      await audioManager.playAnimationSound(AnimationType.ENTITY_MOVEMENT, { 
        isGravityDrop: true, 
        isYourPiece: false 
      });
      expect(playSoundSpy).toHaveBeenCalledWith('drop_opponent_bounce');
    });

    it('should play regular movement sounds when not gravity drop', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      await audioManager.playAnimationSound(AnimationType.ENTITY_MOVEMENT, { 
        isGravityDrop: false, 
        isYourPiece: true 
      });

      expect(playSoundSpy).toHaveBeenCalledWith('place_yours_hard');
    });

    it('should play turn notification sounds based on config', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      // With notifications enabled
      audioManager.updateConfig({ enableTurnNotifications: true });
      await audioManager.playAnimationSound(AnimationType.TURN_CHANGE, { isYourTurn: true });
      expect(playSoundSpy).toHaveBeenCalledWith('your_turn');

      // With notifications disabled
      playSoundSpy.mockClear();
      audioManager.updateConfig({ enableTurnNotifications: false });
      await audioManager.playAnimationSound(AnimationType.TURN_CHANGE, { isYourTurn: true });
      expect(playSoundSpy).not.toHaveBeenCalled();
    });

    it('should play victory fanfare and confetti for wins', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);

      await audioManager.playAnimationSound(AnimationType.GAME_END, { 
        winner: 'p1', 
        isYou: true 
      });

      expect(playSoundSpy).toHaveBeenCalledWith('victory_fanfare');
      
      // Wait for confetti burst timeout
      await new Promise(resolve => setTimeout(resolve, 600));
      expect(playSoundSpy).toHaveBeenCalledWith('confetti_burst');
    });

    it('should not play action sounds when disabled', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);
      
      audioManager.updateConfig({ enableActionSounds: false });
      
      await audioManager.playAnimationSound(AnimationType.ENTITY_SPAWN, { isYourPiece: true });
      
      expect(playSoundSpy).not.toHaveBeenCalled();
    });
  });

  describe('Placeholder Sound Generation', () => {
    it('should create placeholder sounds', async () => {
      const createBufferSpy = vi.spyOn(mockAudioContext, 'createBuffer');
      
      await audioManager.createPlaceholderSounds();

      // Should create buffers for all placeholder sounds
      expect(createBufferSpy).toHaveBeenCalled();
      
      // Check that sounds are stored
      const sounds = (audioManager as any).sounds;
      expect(sounds.size).toBeGreaterThan(0);
    });

    it('should create distinct tones for different sound types', async () => {
      const buffers: Array<{ name: string; data: Float32Array }> = [];
      
      mockAudioContext.createBuffer.mockImplementation((channels, length) => {
        const buffer = {
          getChannelData: vi.fn(() => {
            const data = new Float32Array(length);
            buffers.push({ name: 'buffer', data });
            return data;
          })
        };
        return buffer;
      });

      await audioManager.createPlaceholderSounds();

      // Should have created multiple distinct buffers
      expect(buffers.length).toBeGreaterThan(10);
      
      // Buffers should have different content (simplified check)
      const firstBuffer = buffers[0].data;
      const differentBuffer = buffers.find(b => 
        b.data.some((val, idx) => Math.abs(val - firstBuffer[idx]) > 0.01)
      );
      expect(differentBuffer).toBeTruthy();
    });
  });

  describe('Sound Preloading', () => {
    it('should preload specified sounds', async () => {
      await audioManager.preloadSounds(['place_yours_soft', 'your_turn']);

      expect(global.fetch).toHaveBeenCalledWith('/sounds/place_yours_soft.mp3');
      expect(global.fetch).toHaveBeenCalledWith('/sounds/your_turn.mp3');
      expect(mockAudioContext.decodeAudioData).toHaveBeenCalledTimes(2);
    });

    it('should not reload already cached sounds', async () => {
      await audioManager.preloadSounds(['place_yours_soft']);
      
      vi.clearAllMocks();
      
      await audioManager.preloadSounds(['place_yours_soft']);
      
      expect(global.fetch).not.toHaveBeenCalled();
    });

    it('should handle preload failures gracefully', async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));
      
      // Should not throw
      await expect(audioManager.preloadSounds(['place_yours_soft'])).resolves.not.toThrow();
    });
  });

  describe('Pitch Adjustment', () => {
    it('should apply pitch adjustments', async () => {
      const source = { 
        buffer: null, 
        connect: vi.fn(), 
        start: vi.fn(),
        playbackRate: { value: 1 }
      };
      
      mockAudioContext.createBufferSource.mockReturnValue(source);

      await audioManager.playSound('place_yours_soft', { pitch: 1.5 });

      expect(source.playbackRate.value).toBe(1.5);
    });
  });

  describe('Configuration Updates', () => {
    it('should update configuration correctly', () => {
      const newConfig = {
        enableSounds: false,
        volume: 0.8,
        enableTurnNotifications: false,
        enableActionSounds: false
      };

      audioManager.updateConfig(newConfig);

      const config = (audioManager as any).config;
      expect(config).toMatchObject(newConfig);
    });

    it('should merge partial config updates', () => {
      audioManager.updateConfig({ volume: 0.3 });
      
      const config = (audioManager as any).config;
      expect(config.volume).toBe(0.3);
      expect(config.enableSounds).toBe(DEFAULT_AUDIO_CONFIG.enableSounds);
    });
  });

  describe('Error Handling', () => {
    it('should handle missing AudioContext gracefully', () => {
      delete (global as any).AudioContext;
      
      // Should not throw
      expect(() => new AudioManager()).not.toThrow();
    });

    it('should handle unknown sound IDs', async () => {
      const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
      
      await audioManager.playSound('unknown_sound');
      
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        expect.stringContaining('Unknown sound: unknown_sound')
      );
    });
  });

  describe('All Animation Types Coverage', () => {
    it('should have sounds for all animation types', async () => {
      const playSoundSpy = vi.spyOn(audioManager, 'playSound').mockResolvedValue(undefined);
      
      const animationTypes = [
        { type: AnimationType.ZONE_TRANSFER, expectedSound: 'card_flip' },
        { type: AnimationType.SELECTION_CHANGE, expectedSound: 'selection' },
        { type: AnimationType.CARD_DEAL, expectedSound: 'card_deal' },
        { type: AnimationType.CARD_FLIP, expectedSound: 'card_flip' },
        { type: AnimationType.CARD_SHUFFLE, expectedSound: 'card_shuffle' },
        { type: AnimationType.CARD_COLLECT, expectedSound: 'card_collect' },
        { type: AnimationType.PIECE_CAPTURE, expectedSound: 'piece_capture' },
        { type: AnimationType.SCORE_CHANGE, expectedSound: 'score_change' },
        { type: AnimationType.SHAKE_ERROR, expectedSound: 'error_feedback' }
      ];

      for (const { type, expectedSound } of animationTypes) {
        playSoundSpy.mockClear();
        await audioManager.playAnimationSound(type, {});
        expect(playSoundSpy).toHaveBeenCalledWith(expectedSound);
      }
    });
  });
});