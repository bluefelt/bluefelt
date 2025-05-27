const API_HOST = import.meta.env.VITE_API_HOST || 'localhost:8000';
const API_PROTOCOL = import.meta.env.VITE_API_PROTOCOL || 'http';
const WS_PROTOCOL = API_PROTOCOL === 'https' ? 'wss' : 'ws';

export const API_BASE_URL = `${API_PROTOCOL}://${API_HOST}`;
export const WS_BASE_URL = `${WS_PROTOCOL}://${API_HOST}`;