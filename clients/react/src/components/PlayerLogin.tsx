import { useState } from "react";
import { usePlayer } from "../context/PlayerContext.tsx";

export default function PlayerLogin() {
  const { player, login } = usePlayer();
  const [username, setUsername] = useState("");

  if (player) return null; // already logged in

  return (
    <form
      onSubmit={e => {
        e.preventDefault();
        if (username.trim()) login(username.trim());
      }}
    >
      <input
        value={username}
        onChange={e => setUsername(e.target.value)}
        placeholder="Enter your username"
        autoFocus
      />
      <button type="submit" disabled={!username.trim()}>Log In</button>
    </form>
  );
}