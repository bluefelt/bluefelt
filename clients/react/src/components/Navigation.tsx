import { Link, useLocation, useNavigate } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';

export default function Navigation() {
  const { player, logout } = usePlayer();
  const location = useLocation();
  const navigate = useNavigate();

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  // Don't show navigation on login page or if not logged in
  if (!player || location.pathname === '/login') return null;

  const isInLobby = location.pathname.startsWith('/lobby/');

  return (
    <nav className="bg-gray-800 border-b border-gray-700">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center space-x-4">
            <Link to="/" className="text-xl font-bold">
              Bluefelt
            </Link>
            {!isInLobby && (
              <Link
                to="/lobbies"
                className={`px-3 py-2 rounded-md text-sm font-medium ${
                  location.pathname === '/lobbies'
                    ? 'bg-gray-900 text-white'
                    : 'text-gray-300 hover:bg-gray-700 hover:text-white'
                }`}
              >
                Lobbies
              </Link>
            )}
          </div>
          <div className="flex items-center space-x-4">
            <span className="text-gray-300">
              Playing as: <span className="font-medium text-white">{player.username}</span>
            </span>
            <button
              onClick={handleLogout}
              className="text-gray-300 hover:bg-gray-700 hover:text-white px-3 py-2 rounded-md text-sm font-medium"
            >
              Logout
            </button>
          </div>
        </div>
      </div>
    </nav>
  );
}