export type Lobby = {
  id: string;
  game_id: string;
};

export async function getLobbies(): Promise<Lobby[]> {
  const res = await fetch("http://localhost:8000/lobbies");
  if (!res.ok) throw new Error("Failed to fetch lobbies");
  return res.json();
}

export async function createLobby(game_id: string): Promise<Lobby> {
  const res = await fetch("http://localhost:8000/lobbies", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ game_id }),
  });
  if (!res.ok) throw new Error("Failed to create lobby");
  return res.json();
}