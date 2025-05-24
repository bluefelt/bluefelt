import { usePlayer } from "../context/PlayerContext.tsx";

export default function PlayerProfile() {
  const { player, logout } = usePlayer();
  if (!player) return null;

  return (
    <div>
      <p>Welcome, <strong>{player.username}</strong>!</p>
      <button onClick={logout}>Log Out</button>
    </div>
  );
}