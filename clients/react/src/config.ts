// Use environment variables if set, otherwise use the current host
const getApiHost = () => {
  if (import.meta.env.VITE_API_HOST) {
    return import.meta.env.VITE_API_HOST;
  }
  
  // If running on localhost, use localhost:8000
  if (window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1') {
    return 'localhost:8000';
  }
  
  // Otherwise, use the same host as the frontend but on port 8000
  return `${window.location.hostname}:8000`;
};

const API_HOST = getApiHost();
const API_PROTOCOL = import.meta.env.VITE_API_PROTOCOL || window.location.protocol.replace(':', '');
const WS_PROTOCOL = API_PROTOCOL === 'https' ? 'wss' : 'ws';

export const API_BASE_URL = `${API_PROTOCOL}://${API_HOST}`;
export const WS_BASE_URL = `${WS_PROTOCOL}://${API_HOST}`;