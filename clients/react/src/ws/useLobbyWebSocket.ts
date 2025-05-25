import { useEffect, useRef, useState, useCallback } from "react";
import { applyPatch } from "fast-json-patch";

export type WSMessage = {
  direction: "sent" | "received";
  content: string;
};

type LobbyState = {
  you?: string;
  state?: any;
  meta?: any;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string
) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const [lobbyState, setLobbyState] = useState<LobbyState>({});
  const lastTickRef = useRef(0);
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
    lastTickRef.current = 0;
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
        lastTickRef.current = 0;
        setLobbyState({
          you: data.you,
          state: data.state,
          meta: data.meta,
        });
      } else if (data.type === "diff") {
        if (data.tick !== lastTickRef.current + 1) {
          sendMessage(JSON.stringify({ type: "getDiffs", from: lastTickRef.current + 1 }));
        }
        setLobbyState(prev => {
          if (!prev.state) return prev;
          const root = { state: prev.state, meta: prev.meta };
          const updated = applyPatch(root, data.patch, true, false).newDocument as any;
          return { ...prev, state: updated.state, meta: updated.meta };
        });
        lastTickRef.current = data.tick;
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