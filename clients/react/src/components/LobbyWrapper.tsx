import { useParams, Outlet } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';
import { LobbyWebSocketProvider } from '../context/LobbyWebSocketContext';

export default function LobbyWrapper() {
  const { lobbyId } = useParams<{ lobbyId: string }>();
  const { player } = usePlayer();

  if (!lobbyId || !player?.username) {
    return <div>Loading...</div>;
  }

  return (
    <LobbyWebSocketProvider lobbyId={lobbyId} username={player.username}>
      <Outlet />
    </LobbyWebSocketProvider>
  );
}