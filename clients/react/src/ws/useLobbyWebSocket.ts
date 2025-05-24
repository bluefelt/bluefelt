import { useEffect, useRef, useState, useCallback } from "react";

export type WSMessage = {
  direction: "sent" | "received";
  content: string;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string
) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const wsRef = useRef<WebSocket | null>(null);

  // send a string message
  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
    }
  }, []);

  useEffect(() => {
    const url = `ws://localhost:8000/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(playerId)}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      // optionally: set connection status
    };

    ws.onmessage = (event) => {
      setMessages(msgs => [{ direction: "received", content: event.data }, ...msgs]);
    };

    ws.onerror = (event) => {
      setMessages(msgs => [
        ...msgs,
        { direction: "received", content: "[WebSocket error]: " + event }
      ]);
    };

    ws.onclose = () => {
      // optionally: set connection status
    };

    return () => {
      ws.close();
    }
  }, [lobbyId, playerId]);

  return { messages, sendMessage };
}