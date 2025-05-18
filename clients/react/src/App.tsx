import { Routes, Route, useParams } from 'react-router-dom';
import LobbyList from './pages/LobbyList';
import LobbyRoom from './pages/LobbyRoom';
import LobbyCreate from './pages/LobbyCreate';

export default function App() {
  return (
    <Routes>
      {/* home = lobby directory */}
      <Route path="/" element={<LobbyList />} />

      {/* create lobby */}
      <Route path="/create" element={<LobbyCreate />} />

      {/* dynamic lobby room */}
      <Route path="/lobbies/:id" element={<LobbyRoomWrapper />} />

      {/* catch-all (optional) */}
      <Route path="*" element={<p className="p-4">404 Not Found</p>} />
    </Routes>
  );
}

/* Extract :id from the URL and pass it down */
function LobbyRoomWrapper() {
  const { id } = useParams<{ id: string }>();
  return id ? <LobbyRoom id={id} /> : <p className="p-4">Missing lobby ID</p>;
}
