import { useEffect, useState } from "react";
import { getLobbies } from "../api/lobbies.ts";
import type { Lobby } from "../api/lobbies.ts";
import CreateLobbyDialog from "./CreateLobbyDialog.tsx";

type Props = {
  onLobbySelected: (lobbyId: string) => void;
};

export default function LobbiesList({ onLobbySelected }: Props) {
  const [lobbies, setLobbies] = useState<Lobby[]>([]);
  const [showDialog, setShowDialog] = useState(false);

  const refresh = () => getLobbies().then(setLobbies);

  useEffect(() => {
    refresh();
    // TODO: Better handling of updates, e.g. using WebSocket or polling
  }, []);

  return (
    <div>
      <h2>Lobbies</h2>
      <ul>
        {lobbies.map((lobby) => (
          <li key={lobby.id}>
            Lobby {lobby.id} (Game: {lobby.game_id}){" "}
            <button onClick={() => onLobbySelected(lobby.id)}>View</button>
          </li>
        ))}
      </ul>
      <button onClick={() => setShowDialog(true)}>Create Lobby</button>
      {showDialog && (
        <CreateLobbyDialog
          onCreated={(lobby) => {
            setShowDialog(false);
            refresh();
            onLobbySelected(lobby.id);
          }}
          onCancel={() => setShowDialog(false)}
        />
      )}
    </div>
  )
}