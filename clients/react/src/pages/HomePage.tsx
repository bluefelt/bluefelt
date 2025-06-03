import { useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';

export default function HomePage() {
  const { player } = usePlayer();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    // Preserve any navigation state (like error messages)
    const navigationState = location.state;
    
    if (player) {
      navigate('/lobbies', { replace: true, state: navigationState });
    } else {
      navigate('/login', { replace: true, state: navigationState });
    }
  }, [player, navigate, location.state]);

  return (
    <div className="min-h-screen bg-gray-900 text-white flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-4xl font-bold mb-4">Bluefelt</h1>
        <p className="text-gray-400">Redirecting...</p>
      </div>
    </div>
  );
}