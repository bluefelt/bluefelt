// Centralized logging utility with environment-based control
const isDevelopment = import.meta.env.DEV;
const isDebugEnabled = localStorage.getItem('debug') === 'true';

export const logger = {
  debug: (...args: any[]) => {
    if (isDevelopment && isDebugEnabled) {
      console.log('[DEBUG]', ...args);
    }
  },
  
  info: (...args: any[]) => {
    if (isDevelopment) {
      console.info('[INFO]', ...args);
    }
  },
  
  warn: (...args: any[]) => {
    console.warn('[WARN]', ...args);
  },
  
  error: (...args: any[]) => {
    console.error('[ERROR]', ...args);
  },
  
  // Special debug categories that can be toggled
  websocket: (...args: any[]) => {
    if (isDevelopment && localStorage.getItem('debug:websocket') === 'true') {
      console.log('[WS]', ...args);
    }
  },
  
  gameState: (...args: any[]) => {
    if (isDevelopment && localStorage.getItem('debug:gameState') === 'true') {
      console.log('[GAME]', ...args);
    }
  }
};

// Helper to enable debug logging
export const enableDebug = (category?: string) => {
  if (category) {
    localStorage.setItem(`debug:${category}`, 'true');
  } else {
    localStorage.setItem('debug', 'true');
  }
};

// Helper to disable debug logging
export const disableDebug = (category?: string) => {
  if (category) {
    localStorage.removeItem(`debug:${category}`);
  } else {
    localStorage.removeItem('debug');
  }
};