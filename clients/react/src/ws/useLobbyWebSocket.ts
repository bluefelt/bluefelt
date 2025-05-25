import { useState } from "react";
import { applyPatch } from "fast-json-patch";
import { useWebSocket, WSMessage } from "./useWebSocket";

type LobbyState = {
  bundleMeta?: any;
  state?: any;
};

export function useLobbyWebSocket(
  lobbyId: string,
  playerId: string
) {
  const [lobbyState, setLobbyState] = useState<LobbyState>({});
  const url = `ws://localhost:8000/lobbies/${lobbyId}/ws?player_id=${encodeURIComponent(playerId)}`;

  const { messages, sendMessage } = useWebSocket(url, dataStr => {
    let data: any;
    try {
      data = JSON.parse(dataStr);
    } catch {
      return;
    }

    if (data.type === "welcome") {
      setLobbyState({
        bundleMeta: data.bundleMeta,
        state: data.initialState,
      });
    } else if (data.diff && Array.isArray(data.diff)) {
      setLobbyState(prev => {
        if (!prev.state) return prev;
        const nextState = applyPatch({ ...prev.state }, data.diff, true, false).newDocument;
        return { ...prev, state: nextState };
      });
    }
  });

  return { messages, sendMessage, lobbyState };
}
