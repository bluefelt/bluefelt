import { usePlayer } from "../context/PlayerContext";
import { useLobbyWebSocket } from "../ws/useLobbyWebSocket";
import { useLobbiesWebSocket } from "../ws/useLobbiesWebSocket";
import { useState, useEffect } from "react";
import Board from "./Board";
import TurnIndicator from "./TurnIndicator";
import { getLobby } from "../api/lobbies";

import type { GameManifest } from "../api/games";

type Props = {
  lobbyId: string;
  onLeave: () => void;
};

export default function LobbyView({ lobbyId, onLeave }: Props) {
  const { player } = usePlayer();
  const { lobbyState, joinLobby, leaveLobby, startGame } = useLobbyWebSocket(lobbyId, player!.username, false);
  const joined = lobbyState.you && lobbyState.you !== "spectator";
  const [lobbyInfo, setLobbyInfo] = useState<{
    id: string;
    game_id: string;
    players: string[];
    started: boolean;
    manifest: GameManifest;
  } | null>(null);

  useLobbiesWebSocket((lobbies) => {
    const lobby = lobbies.find((l) => l.id === lobbyId);
    if (lobby) {
      setLobbyInfo((info) => (info ? { ...info, ...lobby } : info));
    }
  });

  useEffect(() => {
    getLobby(lobbyId).then(setLobbyInfo).catch(() => {});
  }, [lobbyId]);

  const canStart =
    lobbyInfo &&
    lobbyInfo.players.length >= lobbyInfo.manifest.metadata.players.min &&
    lobbyInfo.players.length <= lobbyInfo.manifest.metadata.players.max;

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl space-y-6">
      <div className="flex justify-between mb-4">
        <h2 className="text-2xl font-bold">Lobby {lobbyId}</h2>
        <button onClick={onLeave} className="btn btn-secondary">Back to Lobbies</button>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <div className="card space-y-4">
          <h3 className="text-xl font-semibold">Lobby Info</h3>
          <p><strong>ID:</strong> {lobbyId}</p>
          <p><strong>Players:</strong> {lobbyInfo ? lobbyInfo.players.join(", ") || "None" : "Loading..."}</p>
          {joined ? (
            <button onClick={leaveLobby} className="btn btn-secondary">Leave Lobby</button>
          ) : (
            <button onClick={joinLobby} className="btn btn-primary">Join Lobby</button>
          )}
        </div>

        <div className="card space-y-4">
          <h3 className="text-xl font-semibold">Game Info</h3>
          {lobbyInfo ? (
            <>
              <p><strong>Name:</strong> {lobbyInfo.manifest.metadata.name}</p>
              <p><strong>Author:</strong> {lobbyInfo.manifest.metadata.author}</p>
              <p><strong>Description:</strong> {lobbyInfo.manifest.metadata.description}</p>
              <p><strong>Version:</strong> {lobbyInfo.manifest.version}</p>
              <p><strong>Spec Version:</strong> {lobbyInfo.manifest.specVersion}</p>
              <p><strong>Players:</strong> {lobbyInfo.manifest.metadata.players.min} - {lobbyInfo.manifest.metadata.players.max}</p>
              <button onClick={startGame} disabled={!canStart} className="btn btn-primary">
                Start Game
              </button>
              {!canStart && (
                <p className="text-sm text-gray-400">
                  Need {lobbyInfo.manifest.metadata.players.min} to {lobbyInfo.manifest.metadata.players.max} players to start.
                </p>
              )}
            </>
          ) : (
            <p>Loading...</p>
          )}
        </div>
      </div>

      {lobbyState.started && (
        <div className="card space-y-4">
          <TurnIndicator
            you={lobbyState.you}
            turn={lobbyState.state?.turn}
            players={lobbyState.state?.players}
          />
          <Board board={lobbyState.state?.zones?.board ?? []} />
        </div>
      )}
    </div>
  );
}
