import { usePlayer } from "../context/PlayerContext.tsx";

export default function PlayerProfile() {
  const { player } = usePlayer();
  if (!player) return null;

  return (
    <div>
      <h2 className="text-xl font-semibold mb-1">Welcome back!</h2>
      <p className="text-gray-300">Playing as <strong className="text-blue-400">{player.username}</strong></p>
    </div>
  );
}