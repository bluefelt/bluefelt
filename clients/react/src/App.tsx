import { useState } from "react";
import { PlayerProvider, usePlayer } from "./context/PlayerContext.tsx";
import PlayerLogin from "./components/PlayerLogin.tsx";
import PlayerProfile from "./components/PlayerProfile.tsx";
import LobbiesList from "./components/LobbiesList.tsx";
import LobbyView from "./components/LobbyView.tsx";

function MainApp() {
  const { player } = usePlayer();
  const [lobbyId, setLobbyId] = useState<string | null>(null);

  if (!player) return <PlayerLogin />;
  if (lobbyId)
    return <LobbyView lobbyId={lobbyId} onLeave={() => setLobbyId(null)} />;

  return (
    <div>
      <PlayerProfile />
      <LobbiesList onLobbySelected={setLobbyId} />
    </div>
  )
}

export default function App() {
  return (
    <PlayerProvider>
      <MainApp />
    </PlayerProvider>
  );
}