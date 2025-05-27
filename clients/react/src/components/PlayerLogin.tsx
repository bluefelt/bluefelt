import { useState } from "react";
import { usePlayer } from "../context/PlayerContext";

export default function PlayerLogin() {
  const { player, login } = usePlayer();
  const [username, setUsername] = useState("");

  if (player) return null; // already logged in

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="card w-full max-w-md">
        <h1 className="text-2xl font-bold mb-6 text-center">Welcome to Bluefelt</h1>
        <form
          className="space-y-4"
          onSubmit={e => {
            e.preventDefault();
            if (username.trim()) login(username.trim());
          }}
        >
          <div>
            <input
              className="input w-full"
              value={username}
              onChange={e => setUsername(e.target.value)}
              placeholder="Enter your username"
              autoFocus
            />
          </div>
          <button
            type="submit"
            disabled={!username.trim()}
            className="btn btn-primary w-full disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Log In
          </button>
        </form>
      </div>
    </div>
  );
}