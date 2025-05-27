import { useParams, useNavigate } from 'react-router-dom';
import GameView from '../components/GameView';
import ProtectedRoute from '../components/ProtectedRoute';

export default function LobbyPage() {
  const { lobbyId } = useParams<{ lobbyId: string }>();
  const navigate = useNavigate();

  if (!lobbyId) {
    navigate('/lobbies', { replace: true });
    return null;
  }

  const handleLeaveLobby = () => {
    navigate('/lobbies');
  };

  return (
    <ProtectedRoute>
      <GameView lobbyId={lobbyId} onLeave={handleLeaveLobby} />
    </ProtectedRoute>
  );
}