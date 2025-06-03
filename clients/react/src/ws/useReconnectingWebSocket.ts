import { useEffect, useRef, useCallback, useState } from 'react';

export type WSMessage = {
  direction: 'sent' | 'received';
  content: string;
  timestamp: number;
};

export type WebSocketState = 'connecting' | 'connected' | 'disconnected' | 'error';

interface UseReconnectingWebSocketOptions {
  reconnectAttempts?: number;
  reconnectInterval?: number;
  maxReconnectInterval?: number;
  reconnectDecay?: number;
  onOpen?: () => void;
  onClose?: () => void;
  onError?: (error: Event) => void;
  shouldReconnect?: boolean;
}

export function useReconnectingWebSocket(
  url: string,
  onMessage: (data: string) => void,
  options: UseReconnectingWebSocketOptions = {}
) {
  const {
    reconnectAttempts = Infinity,
    reconnectInterval = 1000,
    maxReconnectInterval = 30000,
    reconnectDecay = 1.5,
    onOpen,
    onClose,
    onError,
    shouldReconnect = true,
  } = options;

  const [messages, setMessages] = useState<WSMessage[]>([]);
  const [state, setState] = useState<WebSocketState>('connecting');
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectCountRef = useRef(0);
  const reconnectTimeoutRef = useRef<number | undefined>(undefined);
  const urlRef = useRef(url);
  const onMessageRef = useRef(onMessage);

  // Update refs when dependencies change
  useEffect(() => {
    urlRef.current = url;
  }, [url]);

  useEffect(() => {
    onMessageRef.current = onMessage;
  }, [onMessage]);

  const connect = useCallback(() => {
    // Don't create a new connection if one already exists
    if (wsRef.current && (wsRef.current.readyState === WebSocket.CONNECTING || 
                         wsRef.current.readyState === WebSocket.OPEN)) {
      return;
    }
    
    try {
      setState('connecting');
      const ws = new WebSocket(urlRef.current);
      wsRef.current = ws;

      ws.onopen = () => {
        setState('connected');
        reconnectCountRef.current = 0;
        onOpen?.();
      };

      ws.onclose = (event) => {
        setState('disconnected');
        wsRef.current = null;
        onClose?.();

        // Don't reconnect if we received a close code indicating we shouldn't
        // 1000 = Normal closure
        // 1001 = Going away
        // 1008 = Policy violation (we'll use this for "lobby doesn't exist")
        const shouldNotReconnect = [1000, 1001, 1008].includes(event.code);
        
        if (shouldReconnect && !shouldNotReconnect && reconnectCountRef.current < reconnectAttempts) {
          const timeout = Math.min(
            reconnectInterval * Math.pow(reconnectDecay, reconnectCountRef.current),
            maxReconnectInterval
          );
          
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectCountRef.current++;
            connect();
          }, timeout);
        }
      };

      ws.onerror = (error) => {
        setState('error');
        onError?.(error);
      };

      ws.onmessage = (event) => {
        const now = Date.now();
        const msgId = Math.random().toString(36).substr(2, 9);
        setMessages((msgs) => [
          { direction: 'received', content: event.data, timestamp: now },
          ...msgs,
        ]);
        onMessageRef.current(event.data);
      };
    } catch (error) {
      setState('error');
      console.error('WebSocket connection error:', error);
    }
  }, [shouldReconnect, reconnectAttempts, reconnectInterval, maxReconnectInterval, reconnectDecay, onOpen, onClose, onError]);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      const now = Date.now();
      setMessages((msgs) => [
        { direction: 'sent', content, timestamp: now },
        ...msgs,
      ]);
      return true;
    }
    return false;
  }, []);

  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = undefined;
    }
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    setState('disconnected');
  }, []);

  // Connect on mount only
  useEffect(() => {
    let mounted = true;
    
    // Small delay to ensure cleanup from StrictMode double-mount
    const timer = setTimeout(() => {
      if (mounted) {
        connect();
      }
    }, 10);
    
    return () => {
      mounted = false;
      clearTimeout(timer);
      disconnect();
    };
    // We only want this to run once on mount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Reconnect if URL changes
  useEffect(() => {
    // Skip first mount
    if (!wsRef.current) return;
    
    // Only reconnect if URL actually changed
    if (urlRef.current !== url) {
      urlRef.current = url;
      disconnect();
      connect();
    }
  }, [url, disconnect, connect]);

  return {
    messages,
    sendMessage,
    state,
    connected: state === 'connected',
    disconnect,
    reconnect: connect,
  };
}