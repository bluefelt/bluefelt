import { useEffect } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import PlayerLogin from '../components/PlayerLogin';

export default function LoginPage() {
  const { player } = usePlayer();
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (player) {
      // Check if we have a redirect location from before login
      const from = location.state?.from?.pathname || '/lobbies';
      navigate(from, { replace: true });
    }
  }, [player, navigate, location]);

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <PlayerLogin />
    </div>
  );
}