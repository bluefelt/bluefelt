// Use environment variables if set, otherwise use the current host
const getApiHost = () => {
  if (import.meta.env.VITE_API_HOST) {
    return import.meta.env.VITE_API_HOST;
  }
  
  // If running on localhost, use 127.0.0.1:8000 instead of localhost:8000
  // Some browsers have issues with localhost
  if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    return '127.0.0.1:8000';
  }
  
  // In production, use the same host without port (nginx handles routing)
  return window.location.host;
};

const API_HOST = getApiHost();
const API_PROTOCOL = import.meta.env.VITE_API_PROTOCOL || window.location.protocol.replace(':', '');
const WS_PROTOCOL = API_PROTOCOL === 'https' ? 'wss' : 'ws';

export const API_BASE_URL = `${API_PROTOCOL}://${API_HOST}`;
export const WS_BASE_URL = `${WS_PROTOCOL}://${API_HOST}`;

// Debug logging
console.log('[Config] API_HOST:', API_HOST);
console.log('[Config] API_PROTOCOL:', API_PROTOCOL);
console.log('[Config] API_BASE_URL:', API_BASE_URL);
console.log('[Config] WS_BASE_URL:', WS_BASE_URL);