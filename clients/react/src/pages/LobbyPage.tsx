import { useParams, useNavigate } from 'react-router-dom';
import LobbyView from '../components/LobbyView';
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
      <LobbyView lobbyId={lobbyId} onLeave={handleLeaveLobby} />
    </ProtectedRoute>
  );
}