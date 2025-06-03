import { useNavigate, useLocation } from 'react-router-dom';
import { useEffect, useState } from 'react';
import LobbiesList from '../components/LobbiesList';
import ProtectedRoute from '../components/ProtectedRoute';

export default function LobbiesPage() {
  const navigate = useNavigate();
  const location = useLocation();
  const [message, setMessage] = useState<string | null>(null);

  // Check for messages passed via navigation state
  useEffect(() => {
    if (location.state && (location.state as any).message) {
      setMessage((location.state as any).message);
      // Clear the message from location state
      navigate(location.pathname, { replace: true, state: {} });
      // Auto-dismiss message after 10 seconds
      const timer = setTimeout(() => setMessage(null), 10000);
      return () => clearTimeout(timer);
    }
  }, [location, navigate]);

  const handleLobbySelected = (lobbyId: string) => {
    navigate(`/lobby/${lobbyId}`);
  };

  return (
    <ProtectedRoute>
      <div className="container mx-auto px-4 py-8 max-w-4xl">
        <div className="space-y-8">
          {message && (
            <div className="bg-yellow-900 bg-opacity-50 border border-yellow-600 text-yellow-200 p-4 rounded-lg flex items-center justify-between">
              <p>{message}</p>
              <button 
                onClick={() => setMessage(null)}
                className="ml-4 text-yellow-200 hover:text-white"
              >
                ✕
              </button>
            </div>
          )}
          <div className="card">
            <LobbiesList onLobbySelected={handleLobbySelected} />
          </div>
        </div>
      </div>
    </ProtectedRoute>
  );
}