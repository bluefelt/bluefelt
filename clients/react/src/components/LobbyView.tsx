import { usePlayer } from "../context/PlayerContext";
import { useLobbyWebSocket } from "../ws/useLobbyWebSocket";
import { useState } from "react";

type Props = {
  lobbyId: string;
  onLeave: () => void;
};

export default function LobbyView({ lobbyId, onLeave }: Props) {
  const { player } = usePlayer();
  const [input, setInput] = useState("");
  const { messages, sendMessage, lobbyState } = useLobbyWebSocket(lobbyId, player!.username);

  return (
    <div>
      <h2>Lobby {lobbyId}</h2>
      <button onClick={onLeave}>Leave Lobby</button>
      <div style={{ marginTop: 32 }}>
        <h3>Current Game State</h3>
        <pre style={{ background: "#eee", padding: 12 }}>
          {JSON.stringify(lobbyState, null, 2)}
        </pre>
      </div>
      <form
        onSubmit={e => {
          e.preventDefault();
          if (input.trim()) {
            sendMessage(input);
            setInput("");
          }
        }}
        style={{ marginTop: 24 }}
      >
        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          style={{ width: "60%" }}
          placeholder="Type a JSON message to send"
        />
        <button type="submit" style={{ marginLeft: 8 }}>Send</button>
      </form>
      <div style={{ display: "flex", gap: 16, marginTop: 24 }}>
        <div style={{ flex: 1 }}>
          <h3>Received</h3>
          <ul style={{ minHeight: 100, background: "#f8f8f8", padding: 8 }}>
            {messages.filter(m => m.direction === "received").map((m, i) => (
              <li key={i} style={{ wordBreak: "break-all" }}>{m.content}</li>
            ))}
          </ul>
        </div>
        <div style={{ flex: 1 }}>
          <h3>Sent</h3>
          <ul style={{ minHeight: 100, background: "#f8f8f8", padding: 8 }}>
            {messages.filter(m => m.direction === "sent").map((m, i) => (
              <li key={i} style={{ wordBreak: "break-all" }}>{m.content}</li>
            ))}
          </ul>
        </div>
      </div>
    </div>
  );
}
