import { useState } from "react";
import { PlayerProvider, usePlayer } from "./context/PlayerContext.tsx";
import PlayerLogin from "./components/PlayerLogin.tsx";
import PlayerProfile from "./components/PlayerProfile.tsx";
import LobbiesList from "./components/LobbiesList.tsx";
import LobbyView from "./components/LobbyView.tsx";
import "./index.css";

function MainApp() {
  const { player } = usePlayer();
  const [lobbyId, setLobbyId] = useState<string | null>(null);

  if (!player) return <PlayerLogin />;
  if (lobbyId)
    return <LobbyView lobbyId={lobbyId} onLeave={() => setLobbyId(null)} />;

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl">
      <div className="space-y-8">
        <div className="card">
          <PlayerProfile />
        </div>
        <div className="card">
          <LobbiesList onLobbySelected={setLobbyId} />
        </div>
      </div>
    </div>
  );
}

export default function App() {
  return (
    <PlayerProvider>
      <div className="min-h-screen bg-gray-900 text-white">
        <MainApp />
      </div>
    </PlayerProvider>
  );
}