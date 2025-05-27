import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import { createLobby } from '../api/lobbies';
import { getGames, getGameManifest } from '../api/games';
import type { Game, GameManifest } from '../api/games';
import ProtectedRoute from '../components/ProtectedRoute';

export default function CreateLobbyPage() {
  const navigate = useNavigate();
  const { player } = usePlayer();
  const [gameManifests, setGameManifests] = useState<GameManifest[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadGames() {
      try {
        const games = await getGames();
        const manifests = await Promise.all(
          games.map(game => getGameManifest(game.id))
        );
        setGameManifests(manifests);
      } catch (err) {
        console.error('Failed to load games:', err);
        setError('Failed to load games');
      } finally {
        setLoading(false);
      }
    }
    loadGames();
  }, []);

  const handleCreateLobby = async (gameId: string) => {
    if (!player) return;
    
    setCreating(gameId);
    setError(null);
    
    try {
      const lobby = await createLobby(gameId, player.username);
      navigate(`/lobby/${lobby.id}`);
    } catch (err) {
      console.error('Failed to create lobby:', err);
      setError('Failed to create lobby. Please try again.');
      setCreating(null);
    }
  };

  if (loading) {
    return (
      <ProtectedRoute>
        <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
          <p className="text-gray-400">Loading games...</p>
        </div>
      </ProtectedRoute>
    );
  }

  return (
    <ProtectedRoute>
      <div className="min-h-screen bg-gray-900 text-white">
        <div className="container mx-auto px-4 py-8">
          {/* Header */}
          <div className="mb-8">
            <h1 className="text-3xl font-bold mb-2">Create a New Lobby</h1>
            <p className="text-gray-400">Choose a game to start playing</p>
          </div>

          {/* Error message */}
          {error && (
            <div className="mb-6 p-4 bg-red-900 bg-opacity-50 border border-red-700 rounded-lg text-red-300">
              {error}
            </div>
          )}

          {/* Games grid */}
          {gameManifests.length === 0 ? (
            <div className="text-center py-12">
              <p className="text-gray-400">No games available</p>
            </div>
          ) : (
            <div className="grid gap-6 md:grid-cols-2 lg:grid-cols-3">
              {gameManifests.map(game => (
                <div 
                  key={game.gameId} 
                  className="bg-gray-800 rounded-lg p-6 flex flex-col hover:bg-gray-750 transition-colors"
                >
                  {/* Game header */}
                  <div className="mb-4">
                    <h3 className="text-xl font-semibold mb-1">{game.metadata.name}</h3>
                    <p className="text-sm text-gray-400">by {game.metadata.author}</p>
                  </div>

                  {/* Game description */}
                  <p className="text-gray-300 mb-4 flex-grow">
                    {game.metadata.description}
                  </p>

                  {/* Game details */}
                  <div className="space-y-2 mb-6 text-sm">
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Players:</span>
                      <span className="text-white">
                        {game.metadata.players.min === game.metadata.players.max 
                          ? game.metadata.players.min 
                          : `${game.metadata.players.min}-${game.metadata.players.max}`}
                      </span>
                    </div>
                    <div className="flex items-center justify-between">
                      <span className="text-gray-400">Version:</span>
                      <span className="text-white">{game.version}</span>
                    </div>
                  </div>

                  {/* Create button */}
                  <button
                    onClick={() => handleCreateLobby(game.gameId)}
                    disabled={creating !== null}
                    className={`w-full py-3 px-4 rounded-lg font-medium transition-all ${
                      creating === game.gameId
                        ? 'bg-blue-600 text-white cursor-wait'
                        : creating !== null
                        ? 'bg-gray-700 text-gray-400 cursor-not-allowed'
                        : 'bg-yellow-600 hover:bg-yellow-700 text-black'
                    }`}
                  >
                    {creating === game.gameId ? 'Creating...' : 'Create Lobby'}
                  </button>
                </div>
              ))}
            </div>
          )}

          {/* Back button */}
          <div className="mt-8 text-center">
            <button
              onClick={() => navigate('/lobbies')}
              className="text-gray-400 hover:text-white transition-colors"
            >
              ← Back to Lobbies
            </button>
          </div>
        </div>
      </div>
    </ProtectedRoute>
  );
}