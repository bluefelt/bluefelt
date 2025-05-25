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
  const pendingRef = useRef<string[]>([]);
  const onMessageRef = useRef<(data: string) => void>(() => {});

  useEffect(() => {
    onMessageRef.current = onMessage;
  }, [onMessage]);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
    } else {
      pendingRef.current.push(content);
    }
    setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
  }, []);

  useEffect(() => {
    setMessages([]);
    let ws: WebSocket | null = null;
    let reconnectTimer: number;
    let shouldReconnect = true;

    const connect = () => {
      ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        pendingRef.current.forEach((msg) => ws!.send(msg));
        pendingRef.current = [];
      };

      ws.onmessage = (event) => {
        setMessages((msgs) => [
          { direction: "received", content: event.data },
          ...msgs,
        ]);
        onMessageRef.current(event.data);
      };

      ws.onclose = () => {
        if (shouldReconnect) {
          reconnectTimer = window.setTimeout(connect, 1000);
        }
      };

      ws.onerror = () => {
        ws?.close();
      };
    };

    connect();

    return () => {
      shouldReconnect = false;
      clearTimeout(reconnectTimer);
      ws?.close();
    };
  }, [url]);

  return { messages, sendMessage };
}
