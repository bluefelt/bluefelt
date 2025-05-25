import { useEffect, useRef, useCallback, useState } from "react";

export type WSMessage = {
  direction: "sent" | "received";
  content: string;
};

export function useWebSocket(
  url: string,
  onMessage: (data: string) => void
) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const wsRef = useRef<WebSocket | null>(null);
  const onMessageRef = useRef<(data: string) => void>(() => {});
  const reconnectTimer = useRef<number | null>(null);
  const pendingMessages = useRef<string[]>([]);

  useEffect(() => {
    onMessageRef.current = onMessage;
  }, [onMessage]);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
    } else {
      pendingMessages.current.push(content);
    }
  }, []);

  useEffect(() => {
    let active = true;

    function connect() {
      if (!active) return;
      const ws = new WebSocket(url);
      wsRef.current = ws;
      ws.onopen = () => {
        pendingMessages.current.forEach(m => ws.send(m));
        pendingMessages.current = [];
      };
      ws.onmessage = (event) => {
        setMessages(msgs => [{ direction: "received", content: event.data }, ...msgs]);
        onMessageRef.current(event.data);
      };
      ws.onclose = () => {
        if (active) {
          reconnectTimer.current = window.setTimeout(connect, 1000);
        }
      };
      ws.onerror = () => {
        ws.close();
      };
    }

    setMessages([]);
    connect();

    return () => {
      active = false;
      if (reconnectTimer.current) clearTimeout(reconnectTimer.current);
      wsRef.current?.close();
    };
  }, [url]);

  return { messages, sendMessage };
}
