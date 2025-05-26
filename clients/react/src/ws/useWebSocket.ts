import { useEffect, useRef, useCallback, useState } from 'react';

export type WSMessage = {
  direction: 'sent' | 'received';
  content: string;
};

export function useWebSocket(url: string, onMessage: (data: string) => void) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const [connected, setConnected] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const onMessageRef = useRef<(data: string) => void>(() => {});

  useEffect(() => {
    onMessageRef.current = onMessage;
  }, [onMessage]);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages((msgs) => [{ direction: 'sent', content }, ...msgs]);
    }
  }, []);

  useEffect(() => {
    setMessages([]);
    setConnected(false);
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => setConnected(true);
    ws.onclose = () => setConnected(false);
    ws.onerror = () => setConnected(false);

    ws.onmessage = (event) => {
      setMessages((msgs) => [
        { direction: 'received', content: event.data },
        ...msgs,
      ]);
      onMessageRef.current(event.data);
    };

    return () => {
      ws.close();
      setConnected(false);
    };
  }, [url]);

  return { messages, sendMessage, connected };
}
