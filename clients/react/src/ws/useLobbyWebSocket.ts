import { useEffect, useRef, useState, useCallback } from "react";
import { applyPatch } from "fast-json-patch";

export type WSMessage = {
  direction: "sent" | "received";
  content: string;
};

type LobbyState = {
  bundleMeta?: any;
  state?: any;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string
) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const [lobbyState, setLobbyState] = useState<LobbyState>({});
  const wsRef = useRef<WebSocket | null>(null);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
    }
  }, []);

  useEffect(() => {
    setMessages([]);
    setLobbyState({});
    const url = `ws://localhost:8000/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(playerId)}`;
    const ws = new WebSocket(url);
    wsRef.current = ws;

    ws.onopen = () => {
      // optionally: set connection status
    };

    ws.onmessage = (event) => {
      setMessages(msgs => [{ direction: "received", content: event.data }, ...msgs]);
      let data: any;
      try {
        data = JSON.parse(event.data);
      } catch (err) {
        return;
      }

      if (data.type === "welcome") {
        setLobbyState({
          bundleMeta: data.bundleMeta,
          state: data.initialState,
        });
      } else if (data.diff && Array.isArray(data.diff)) {
        setLobbyState((prev) => {
          if (!prev.state) return prev; // not initialized yet
          const nextState = applyPatch({ ...prev.state }, data.diff, true, false).newDocument;
          return { ...prev, state: nextState };
        });
      }
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

  return { messages, sendMessage, lobbyState };
}