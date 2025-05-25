export type Lobby = {
  id: string;
  game_id: string;
  name: string;
  players: string[];
  started: boolean;
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

export async function getLobby(id: string): Promise<Lobby & { manifest: any }> {
  const res = await fetch(`http://localhost:8000/lobbies/${id}`);
  if (!res.ok) throw new Error("Failed to fetch lobby");
  return res.json();
}