import { vi } from 'vitest';
import React from 'react';

// Mock AnimationContext
export const mockAnimationContext = {
  useAnimationsEnabled: vi.fn(() => true),
  useAnimation: vi.fn(() => ({
    state: { isAnimating: false, config: { enableAnimations: true } },
    updateConfig: vi.fn(),
    addAnimation: vi.fn(),
    removeAnimation: vi.fn(),
    clearQueue: vi.fn(),
    isAnimating: false
  })),
  AnimationProvider: ({ children }: { children: React.ReactNode }) => React.createElement('div', null, children)
};

// Mock PlayerPreferencesContext
export const mockPlayerPreferencesContext = {
  usePlayerPreferences: vi.fn(() => ({
    preferences: {
      cardStyle: 'default',
      colorScheme: 'default',
      tokenStyle: 'default'
    },
    updatePreferences: vi.fn(),
    getPlayerCardStyle: vi.fn(() => 'default'),
    getPlayerColorScheme: vi.fn(() => 'default'),
    getPlayerToken: vi.fn(() => 'default')
  })),
  PlayerPreferencesProvider: ({ children }: { children: React.ReactNode }) => React.createElement('div', null, children)
};

// Mock PlayerContext
export const mockPlayerContext = {
  usePlayer: vi.fn(() => ({
    player: { username: 'testuser', color: '#FF0000' }
  })),
  PlayerProvider: ({ children }: { children: React.ReactNode }) => React.createElement('div', null, children)
};

// Mock useGameActions hook
export const mockUseGameActions = {
  useGameActions: vi.fn(() => ({
    sendAction: vi.fn(),
    canPerformAction: vi.fn(() => true),
    getAvailableActions: vi.fn(() => []),
    isMyTurn: vi.fn(() => true)
  }))
};