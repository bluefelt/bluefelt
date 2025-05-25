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

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
    }
  }, []);

  useEffect(() => {
    setMessages([]);
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onmessage = (event) => {
      setMessages(msgs => [{ direction: "received", content: event.data }, ...msgs]);
      onMessage(event.data);
    };

    return () => {
      ws.close();
    };
  }, [url, onMessage]);

  return { messages, sendMessage };
}
