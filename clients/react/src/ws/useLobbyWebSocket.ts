import { useEffect, useRef, useState, useCallback } from "react";
import { useLobbyStore } from "../store/lobbyStore";
import { shallow } from "zustand/shallow";

export type WSMessage = {
  direction: "sent" | "received";
  content: string;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string
) {
  const [messages, setMessages] = useState<WSMessage[]>([]);
  const lobbyState = useLobbyStore(
    (s) => ({ bundleMeta: s.bundleMeta, state: s.state }),
    shallow
  );
  const setInitialState = useLobbyStore((s) => s.setInitialState);
  const applyDiff = useLobbyStore((s) => s.applyDiff);
  const resetLobby = useLobbyStore((s) => s.reset);
  const wsRef = useRef<WebSocket | null>(null);

  const sendMessage = useCallback((content: string) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      wsRef.current.send(content);
      setMessages(msgs => [{ direction: "sent", content }, ...msgs]);
    }
  }, []);

  useEffect(() => {
    setMessages([]);
    resetLobby();
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
        setInitialState(data.bundleMeta, data.initialState);
      } else if (data.diff && Array.isArray(data.diff)) {
        applyDiff(data.diff);
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