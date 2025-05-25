import { usePlayer } from "../context/PlayerContext";
import { useLobbyWebSocket } from "../ws/useLobbyWebSocket";
import { useState } from "react";
import Board from "./Board";
import TurnIndicator from "./TurnIndicator";

type Props = {
  lobbyId: string;
  onLeave: () => void;
};

export default function LobbyView({ lobbyId, onLeave }: Props) {
  const { player } = usePlayer();
  const [input, setInput] = useState("");
  const { messages, sendMessage, lobbyState } = useLobbyWebSocket(lobbyId, player!.username);

  return (
    <div className="container mx-auto px-4 py-8 max-w-4xl space-y-6">
      <div className="card">
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-2xl font-bold">Lobby {lobbyId}</h2>
          <button onClick={onLeave} className="btn btn-secondary">
            Leave Lobby
          </button>
        </div>
        
        <div className="space-y-4">
          <TurnIndicator
            you={lobbyState.you}
            turn={lobbyState.state?.turn}
            players={lobbyState.state?.players}
          />
          <Board board={lobbyState.state?.zones?.board ?? []} />
        </div>
      </div>

      <div className="card space-y-4">
        <h3 className="text-xl font-semibold">Current Game State</h3>
        <pre className="bg-gray-700 p-4 rounded-lg overflow-auto text-sm">
          {JSON.stringify(lobbyState, null, 2)}
        </pre>
      </div>

      <div className="card">
        <form
          onSubmit={e => {
            e.preventDefault();
            if (input.trim()) {
              sendMessage(input);
              setInput("");
            }
          }}
          className="space-y-4"
        >
          <div className="flex gap-3">
            <input
              value={input}
              onChange={e => setInput(e.target.value)}
              className="input flex-1"
              placeholder="Type a JSON message to send"
            />
            <button type="submit" className="btn btn-primary">
              Send
            </button>
          </div>
        </form>

        <div className="grid grid-cols-2 gap-6 mt-6">
          <div>
            <h3 className="text-lg font-semibold mb-3">Received</h3>
            <ul className="bg-gray-700 rounded-lg p-4 min-h-[100px] space-y-2 text-sm">
              {messages.filter(m => m.direction === "received").map((m, i) => (
                <li key={i} className="break-all">{m.content}</li>
              ))}
            </ul>
          </div>
          <div>
            <h3 className="text-lg font-semibold mb-3">Sent</h3>
            <ul className="bg-gray-700 rounded-lg p-4 min-h-[100px] space-y-2 text-sm">
              {messages.filter(m => m.direction === "sent").map((m, i) => (
                <li key={i} className="break-all">{m.content}</li>
              ))}
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}
