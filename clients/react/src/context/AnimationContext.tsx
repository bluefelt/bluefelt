/**
 * Animation Context for managing animation configuration and state across the app.
 * 
 * Provides centralized animation settings, state management, and persistence.
 */

import React, { createContext, useContext, useReducer, useEffect, useRef, ReactNode } from 'react';
import { 
  type AnimationConfig,
  type AnimationState,
  type AnimationPlan,
  type PatchOperation,
  DEFAULT_ANIMATION_CONFIG 
} from '../animation/AnimationTypes';
import { AudioManager, type AudioConfig, DEFAULT_AUDIO_CONFIG } from '../animation/AudioManager';
import { AnimationEngine, type AnimationEngineCallbacks } from '../animation/AnimationEngine';

interface AnimationContextType {
  state: AnimationState;
  updateConfig: (config: Partial<AnimationConfig>) => void;
  addAnimation: (animation: AnimationPlan) => void;
  removeAnimation: (animationId: string) => void;
  clearQueue: () => void;
  isAnimating: boolean;
  // Methods expected by tests
  processPatches: (patches: PatchOperation[], gameState: any, playerId: string) => Promise<void>;
  getAnimationStatus: () => { isProcessing: boolean; currentCount: number; queuedCount: number };
  engine: AnimationEngine | null;
}

// Action types for the reducer
type AnimationAction = 
  | { type: 'UPDATE_CONFIG'; payload: Partial<AnimationConfig> }
  | { type: 'ADD_ANIMATION'; payload: AnimationPlan }
  | { type: 'REMOVE_ANIMATION'; payload: string }
  | { type: 'CLEAR_QUEUE' }
  | { type: 'SET_ANIMATING'; payload: boolean }
  | { type: 'LOAD_CONFIG'; payload: AnimationConfig };

const AnimationContext = createContext<AnimationContextType | undefined>(undefined);

// Storage key for persisting animation preferences
const STORAGE_KEY = 'bluefelt-animation-config';

// Load configuration from localStorage with fallback to defaults
const loadStoredConfig = (): AnimationConfig => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_ANIMATION_CONFIG, ...parsed };
    }
  } catch (error) {
    console.warn('[AnimationContext] Failed to load stored config:', error);
  }
  return DEFAULT_ANIMATION_CONFIG;
};

// Save configuration to localStorage
const saveConfig = (config: AnimationConfig): void => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
  } catch (error) {
    console.warn('[AnimationContext] Failed to save config:', error);
  }
};

// Check for user's motion preferences
const getMotionPreference = (): boolean => {
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
};

// Initial state
const createInitialState = (): AnimationState => {
  const config = loadStoredConfig();
  
  // Override with system preference if user hasn't explicitly set it
  if (getMotionPreference()) {
    config.reduceMotion = true;
    config.enableAnimations = false;
  }
  
  return {
    isAnimating: false,
    currentAnimations: new Set(),
    queuedAnimations: [],
    config
  };
};

// Reducer for managing animation state
const animationReducer = (state: AnimationState, action: AnimationAction): AnimationState => {
  switch (action.type) {
    case 'UPDATE_CONFIG': {
      const newConfig = { ...state.config, ...action.payload };
      saveConfig(newConfig);
      return {
        ...state,
        config: newConfig
      };
    }
    
    case 'ADD_ANIMATION': {
      // Don't add if animations are disabled
      if (!state.config.enableAnimations) {
        return state;
      }
      
      // Prevent queue from growing too large
      const queuedAnimations = [...state.queuedAnimations];
      if (queuedAnimations.length >= state.config.maxQueueSize) {
        // Remove oldest animation
        queuedAnimations.shift();
      }
      
      queuedAnimations.push(action.payload);
      
      return {
        ...state,
        queuedAnimations
      };
    }
    
    case 'REMOVE_ANIMATION': {
      const newCurrentAnimations = new Set(state.currentAnimations);
      newCurrentAnimations.delete(action.payload);
      
      return {
        ...state,
        currentAnimations: newCurrentAnimations,
        queuedAnimations: state.queuedAnimations.filter(anim => anim.id !== action.payload),
        isAnimating: newCurrentAnimations.size > 0
      };
    }
    
    case 'CLEAR_QUEUE': {
      return {
        ...state,
        queuedAnimations: []
      };
    }
    
    case 'SET_ANIMATING': {
      return {
        ...state,
        isAnimating: action.payload
      };
    }
    
    case 'LOAD_CONFIG': {
      return {
        ...state,
        config: action.payload
      };
    }
    
    default:
      return state;
  }
};

interface AnimationProviderProps {
  children: ReactNode;
}

export const AnimationProvider: React.FC<AnimationProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(animationReducer, undefined, createInitialState);
  const animationEngineRef = useRef<AnimationEngine | null>(null);
  
  // Initialize animation engine
  useEffect(() => {
    const callbacks: AnimationEngineCallbacks = {
      onAnimationStart: (animation) => {
        dispatch({ type: 'ADD_ANIMATION', payload: animation });
      },
      onAnimationComplete: (result) => {
        dispatch({ type: 'REMOVE_ANIMATION', payload: result.animationId });
      },
      onQueueEmpty: () => {
        dispatch({ type: 'SET_ANIMATING', payload: false });
      }
    };
    
    animationEngineRef.current = new AnimationEngine(callbacks);
  }, []);
  
  // Update animation engine config when state changes
  useEffect(() => {
    if (animationEngineRef.current) {
      animationEngineRef.current.updateAudioConfig(state.config);
    }
  }, [state.config]);
  
  // Listen for motion preference changes
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    
    const handleChange = (e: MediaQueryListEvent) => {
      if (e.matches) {
        dispatch({ 
          type: 'UPDATE_CONFIG', 
          payload: { 
            reduceMotion: true, 
            enableAnimations: false 
          } 
        });
      }
    };
    
    mediaQuery.addEventListener('change', handleChange);
    
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  }, []);
  
  const updateConfig = (config: Partial<AnimationConfig>) => {
    dispatch({ type: 'UPDATE_CONFIG', payload: config });
  };
  
  const addAnimation = (animation: AnimationPlan) => {
    dispatch({ type: 'ADD_ANIMATION', payload: animation });
  };
  
  const removeAnimation = (animationId: string) => {
    dispatch({ type: 'REMOVE_ANIMATION', payload: animationId });
  };
  
  const clearQueue = () => {
    dispatch({ type: 'CLEAR_QUEUE' });
  };
  
  // Implementation of methods expected by tests
  const processPatches = async (patches: PatchOperation[], gameState: any, playerId: string): Promise<void> => {
    if (!animationEngineRef.current || !state.config.enableAnimations) {
      return;
    }
    
    // Process each patch with the animation engine
    for (const patch of patches) {
      try {
        await animationEngineRef.current.processAnimatablePatch(
          patch,
          gameState,
          state.config,
          playerId
        );
      } catch (error) {
        console.error('[AnimationContext] Failed to process patch:', error, patch);
      }
    }
  };
  
  const getAnimationStatus = () => {
    if (!animationEngineRef.current) {
      return { isProcessing: false, currentCount: 0, queuedCount: 0 };
    }
    return animationEngineRef.current.getStatus();
  };
  
  // Create a state object with compatibility for tests expecting 'queue' property
  const compatibleState = {
    ...state,
    queue: state.queuedAnimations // Add compatibility property for tests
  };
  
  const value: AnimationContextType = {
    state: compatibleState,
    updateConfig,
    addAnimation,
    removeAnimation,
    clearQueue,
    isAnimating: state.isAnimating,
    processPatches,
    getAnimationStatus,
    engine: animationEngineRef.current
  };
  
  return (
    <AnimationContext.Provider value={value}>
      {children}
    </AnimationContext.Provider>
  );
};

// Hook for using animation context
export const useAnimation = (): AnimationContextType => {
  const context = useContext(AnimationContext);
  if (context === undefined) {
    throw new Error('useAnimation must be used within an AnimationProvider');
  }
  return context;
};

// Hook for just the animation config (lightweight)
export const useAnimationConfig = (): AnimationConfig => {
  const { state } = useAnimation();
  return state.config;
};

// Hook for checking if animations are enabled and should be shown
export const useAnimationsEnabled = (): boolean => {
  const { state } = useAnimation();
  return state.config.enableAnimations && !state.config.reduceMotion;
};