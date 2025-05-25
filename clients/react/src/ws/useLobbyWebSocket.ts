import { useState } from "react";
import { applyPatch } from "fast-json-patch";
import { useWebSocket, type WSMessage } from "./useWebSocket";

type LobbyState = {
  you?: string;
  meta?: any;
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
        you: data.you,
        meta: data.meta,
        state: data.state,
      });
    } else if (data.type === "diff" && Array.isArray(data.patch)) {
      setLobbyState(prev => {
        const full = { meta: prev.meta, state: prev.state };
        const patched = applyPatch({ ...full }, data.patch, true, false).newDocument as LobbyState;
        return patched;
      });
    }
  });

  return { messages, sendMessage, lobbyState };
}