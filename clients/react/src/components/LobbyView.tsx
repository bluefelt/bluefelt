import { usePlayer } from '../context/PlayerContext';
import { useLobbyWebSocket } from '../ws/useLobbyWebSocket';
import WebSocketStatus from './WebSocketStatus';
import { useState, useEffect } from 'react';
import InteractiveZone from './InteractiveZone';
import TurnIndicator from './TurnIndicator';
import GameEndDisplay from './GameEndDisplay';
import { getLobby } from '../api/lobbies';

import type { GameManifest } from '../api/games';

type Props = {
  lobbyId: string;
  onLeave: () => void;
};

export default function LobbyView({ lobbyId, onLeave }: Props) {
  const { player } = usePlayer();
  const { sendMessage, lobbyState, joinLobby, leaveLobby, startGame, connectionState } = useLobbyWebSocket(lobbyId, player!.username, false);
  const [input, setInput] = useState("");
  const joined = lobbyState.you && lobbyState.you !== "spectator";
  const [lobbyInfo, setLobbyInfo] = useState<{
    id: string;
    game_id: string;
    players: string[];
    started: boolean;
    manifest: GameManifest;
  } | null>(null);

  // Fetch initial lobby info and update when players change
  useEffect(() => {
    getLobby(lobbyId).then(setLobbyInfo).catch(() => {});
  }, [lobbyId]);

  // Update lobby info when we receive state updates
  useEffect(() => {
    if (lobbyState.meta && lobbyState.meta.players) {
      setLobbyInfo(prev => {
        if (!prev) return prev;
        return {
          ...prev,
          players: lobbyState.meta.players,
          started: lobbyState.started || false
        };
      });
    }
  }, [lobbyState.meta, lobbyState.started]);

  const canStart = lobbyInfo &&
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
          <p><strong>Connection:</strong> {connectionState}</p>
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
        <>
          <GameEndDisplay 
            gameStatus={lobbyState.meta?.gameStatus}
            you={lobbyState.you}
            onClose={() => {
              // Optional: could add logic here if needed when modal closes
            }}
          />
          
          <div className="card space-y-4">
            <h3 className="text-xl font-semibold">Game Board</h3>
            <TurnIndicator
              you={lobbyState.you}
              turn={lobbyState.state?.turn}
              players={lobbyState.state?.players}
              gameStatus={lobbyState.meta?.gameStatus}
            />
            {/* Display verb directions if it's the player's turn */}
            {lobbyState.you && lobbyState.you !== 'spectator' && 
             lobbyState.state?.turn === lobbyState.you && 
             lobbyState.meta?.possibleVerbs?.[lobbyState.you] && (
              <div className="space-y-2">
                {lobbyState.meta.possibleVerbs[lobbyState.you].map((groupedVerb, idx) => (
                  <p key={idx} className="text-sm text-blue-400 font-semibold animate-pulse">
                    {groupedVerb.direction}
                  </p>
                ))}
              </div>
            )}
            {/* Render interactive zones */}
            {lobbyState.state?.zones && Object.entries(lobbyState.state.zones).map(([zoneName, zoneData]) => {
              if (!Array.isArray(zoneData)) return null;
              
              const myGroupedVerbs = lobbyState.meta?.possibleVerbs?.[lobbyState.you || ''] || [];
              const isGameEnded = lobbyState.meta?.gameStatus?.state === 'ended';
              const isMyTurn = lobbyState.you && lobbyState.you !== 'spectator' && myGroupedVerbs.length > 0 && !isGameEnded;
              
              if (zoneName === 'board' && myGroupedVerbs.length > 0) {
                console.log('[LobbyView] Zone render:', { 
                  zoneName, 
                  you: lobbyState.you, 
                  myGroupedVerbs, 
                  isMyTurn,
                  turn: lobbyState.state?.turn 
                });
              }
              
              return (
                <InteractiveZone
                  key={zoneName}
                  zoneName={zoneName}
                  zoneData={zoneData as (string | null)[][]}
                  groupedVerbs={myGroupedVerbs}
                  isMyTurn={!!isMyTurn}
                  onAction={(action) => {
                    const message = JSON.stringify({
                      verb: action.verb,
                      args: { row: action.row, col: action.col }
                    });
                    sendMessage(message);
                  }}
                />
              );
            })}
          </div>
          <div className="card space-y-4">
            <h3 className="text-xl font-semibold">Actions</h3>
            <form
              onSubmit={e => {
                e.preventDefault();
                if (input.trim()) {
                  sendMessage(input);
                  setInput("");
                }
              }}
              className="space-y-4"
            >
              <input
                value={input}
                onChange={e => setInput(e.target.value)}
                className="input w-full bg-gray-700 text-white"
                placeholder="Type a JSON message to send"
              />
              <button type="submit" className="btn btn-primary">
                Send
              </button>
            </form>
          </div>
          <div className="card space-y-4">
            <h3 className="text-xl font-semibold">Game State</h3>
            <pre className="bg-gray-700 p-4 rounded-lg overflow-auto text-sm">
              {JSON.stringify(lobbyState, null, 2)}
            </pre>
          </div>
        </>
      )}
      <WebSocketStatus connected={connectionState === 'connected'} state={connectionState} />
    </div>
  );
}