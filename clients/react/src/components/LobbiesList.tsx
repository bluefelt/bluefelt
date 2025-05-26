import { useEffect, useState } from 'react';
import { getLobbies } from '../api/lobbies.ts';
import type { Lobby } from '../api/lobbies.ts';
import { useLobbiesWebSocket } from '../ws/useLobbiesWebSocket';
import WebSocketStatus from './WebSocketStatus';
import CreateLobbyDialog from './CreateLobbyDialog.tsx';
import { usePlayer } from '../context/PlayerContext.tsx';

type Props = {
  onLobbySelected: (lobbyId: string) => void;
};

export default function LobbiesList({ onLobbySelected }: Props) {
  const [lobbies, setLobbies] = useState<Lobby[]>([]);
  const [showDialog, setShowDialog] = useState(false);
  const { player } = usePlayer();

  const refresh = () => getLobbies().then(setLobbies);

  const { connected } = useLobbiesWebSocket(setLobbies);

  useEffect(() => {
    refresh();
  }, []);

  return (
    <>
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold">Available Lobbies</h2>
          <button
            onClick={() => setShowDialog(true)}
            className="btn btn-primary"
          >
            Create Lobby
          </button>
        </div>

        {lobbies.length === 0 ? (
          <p className="text-gray-400 text-center py-8">
            No active lobbies. Create one to get started!
          </p>
        ) : (
          <ul className="space-y-4">
            {lobbies.map((lobby) => (
              <li
                key={lobby.id}
                className="bg-gray-700 rounded-lg p-4 flex justify-between items-center"
              >
                <div>
                  <h3 className="font-medium">{lobby.name}</h3>
                  <p className="text-sm text-gray-400">Game: {lobby.game_id}</p>
                  <p className="text-sm text-gray-400">
                    Players:{' '}
                    {lobby.players
                      .map((p) => (p === player?.username ? `${p} (you)` : p))
                      .join(', ') || 'None'}
                  </p>
                  <p className="text-sm text-gray-400">
                    Status:{' '}
                    {lobby.started
                      ? 'Started'
                      : lobby.players.length >= 2
                        ? 'Not Started'
                        : 'Waiting for Players'}
                  </p>
                </div>
                <button
                  onClick={() => onLobbySelected(lobby.id)}
                  className="btn btn-secondary"
                >
                  Open Lobby
                </button>
              </li>
            ))}
          </ul>
        )}

        {showDialog && (
          <CreateLobbyDialog
            onCreated={(lobby) => {
              setShowDialog(false);
              onLobbySelected(lobby.id);
            }}
            onCancel={() => setShowDialog(false)}
          />
        )}
      </div>
      <WebSocketStatus connected={connected} />
    </>
  );
}
