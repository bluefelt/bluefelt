import { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import PlayerLogin from '../components/PlayerLogin';

export default function LoginPage() {
  const { player } = usePlayer();
  const navigate = useNavigate();
  const location = useLocation();
  const [message, setMessage] = useState<string | null>(null);

  // Check for messages passed via navigation state
  useEffect(() => {
    if (location.state && (location.state as any).message) {
      setMessage((location.state as any).message);
    }
  }, [location]);

  useEffect(() => {
    if (player) {
      // Check if we have a redirect location from before login
      const from = location.state?.from?.pathname || '/lobbies';
      // Preserve any message that was passed through navigation
      const message = location.state?.message;
      navigate(from, { replace: true, state: message ? { message } : undefined });
    }
  }, [player, navigate, location]);

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      {message && (
        <div className="max-w-md mx-auto mt-4 px-4">
          <div className="bg-yellow-900 bg-opacity-50 border border-yellow-600 text-yellow-200 p-4 rounded-lg flex items-center justify-between">
            <p className="text-sm">{message}</p>
            <button 
              onClick={() => setMessage(null)}
              className="ml-4 text-yellow-200 hover:text-white"
            >
              ✕
            </button>
          </div>
        </div>
      )}
      <PlayerLogin />
    </div>
  );
}