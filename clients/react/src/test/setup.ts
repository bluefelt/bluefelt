import { expect, afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'
import * as matchers from '@testing-library/jest-dom/matchers'
import { mockAnimationContext, mockPlayerPreferencesContext, mockPlayerContext, mockUseGameActions } from './mocks'

// extends Vitest's expect method with methods from react-testing-library
expect.extend(matchers)

// runs a cleanup after each test case (e.g. clearing jsdom)
afterEach(() => {
  cleanup()
})

// Global mocks for contexts
vi.mock('../context/AnimationContext', () => mockAnimationContext);
vi.mock('../context/PlayerPreferencesContext', () => mockPlayerPreferencesContext);
vi.mock('../context/PlayerContext', () => mockPlayerContext);

// Mock WebSocket for tests
global.WebSocket = class WebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = WebSocket.CONNECTING;
  url: string;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;

  constructor(url: string) {
    this.url = url;
    // Simulate async connection
    setTimeout(() => {
      this.readyState = WebSocket.OPEN;
      if (this.onopen) {
        this.onopen(new Event('open'));
      }
    }, 0);
  }

  send(data: string): void {
    if (this.readyState !== WebSocket.OPEN) {
      throw new Error('WebSocket is not open');
    }
    // Mock sending - in tests we can spy on this method
  }

  close(): void {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }
};

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => {}
  })
});

// Mock AudioContext for AudioManager
const MockAudioContext = class AudioContext {
  sampleRate = 44100;
  state = 'running';
  
  createGain() {
    return {
      connect: vi.fn(),
      disconnect: vi.fn(),
      gain: { 
        value: 1,
        setValueAtTime: vi.fn(),
        exponentialRampToValueAtTime: vi.fn()
      }
    };
  }
  
  createBufferSource() {
    return {
      connect: vi.fn(),
      disconnect: vi.fn(),
      start: vi.fn(),
      stop: vi.fn(),
      buffer: null,
      playbackRate: { value: 1 }
    };
  }
  
  createBuffer(numberOfChannels: number, length: number, sampleRate: number) {
    // More robust mock that always returns a working buffer
    const channels: Float32Array[] = [];
    for (let i = 0; i < numberOfChannels; i++) {
      channels[i] = new Float32Array(length);
    }
    
    return {
      numberOfChannels,
      length,
      sampleRate,
      getChannelData: (channel: number) => {
        if (channel < 0 || channel >= numberOfChannels) {
          throw new Error('Invalid channel index');
        }
        return channels[channel] || new Float32Array(length);
      },
      duration: length / sampleRate
    };
  }
  
  decodeAudioData() {
    return Promise.resolve(this.createBuffer(1, 1024, 44100));
  }
  
  close() {
    return Promise.resolve();
  }
  
  suspend() {
    return Promise.resolve();
  }
  
  resume() {
    return Promise.resolve();
  }
  
  destination = {
    channelCount: 2,
    channelCountMode: 'explicit',
    channelInterpretation: 'speakers'
  };
};

// Set up the mock for both global and window, and handle different naming conventions
global.AudioContext = MockAudioContext;
global.webkitAudioContext = MockAudioContext;
// @ts-ignore
window.AudioContext = MockAudioContext;
// @ts-ignore  
window.webkitAudioContext = MockAudioContext;