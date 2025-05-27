import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { getLobbies } from '../api/lobbies.ts';
import { useWebSocketContext } from '../context/WebSocketContext.tsx';
import WebSocketStatus from './WebSocketStatus';
import { usePlayer } from '../context/PlayerContext.tsx';

type Props = {
  onLobbySelected: (lobbyId: string) => void;
};

export default function LobbiesList({ onLobbySelected }: Props) {
  const navigate = useNavigate();
  const { player } = usePlayer();
  const { lobbies, lobbiesWS } = useWebSocketContext();

  useEffect(() => {
    // Fetch initial lobbies list
    getLobbies().then(() => {
      // The WebSocket will handle updates
    }).catch(console.error);
  }, []);

  return (
    <>
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold">Available Lobbies</h2>
          <button
            onClick={() => navigate('/create-lobby')}
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
                className="bg-gray-700 rounded-lg p-4 flex justify-between items-center hover:bg-gray-600 transition-colors"
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
                    <span className={lobby.started ? 'text-green-400' : 'text-yellow-400'}>
                      {lobby.started
                        ? 'In Progress'
                        : lobby.players.length >= 2
                          ? 'Ready to Start'
                          : 'Waiting for Players'}
                    </span>
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
      </div>
      <WebSocketStatus connected={lobbiesWS.connected} state={lobbiesWS.state} />
    </>
  );
}