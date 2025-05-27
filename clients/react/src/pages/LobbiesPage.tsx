import { useNavigate } from 'react-router-dom';
import PlayerProfile from '../components/PlayerProfile';
import LobbiesList from '../components/LobbiesList';
import ProtectedRoute from '../components/ProtectedRoute';

export default function LobbiesPage() {
  const navigate = useNavigate();

  const handleLobbySelected = (lobbyId: string) => {
    navigate(`/lobby/${lobbyId}`);
  };

  return (
    <ProtectedRoute>
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        <div className="space-y-8">
          <div className="card">
            <LobbiesList onLobbySelected={handleLobbySelected} />
          </div>
        </div>
      </div>
    </ProtectedRoute>
  );
}