import { API_BASE_URL } from '../config';

export type Lobby = {
  id: string;
  game_id: string;
  name: string;
  players: string[];
  started: boolean;
};

export async function getLobbies(): Promise<Lobby[]> {
  const res = await fetch(`${API_BASE_URL}/lobbies`);
  if (!res.ok) throw new Error("Failed to fetch lobbies");
  return res.json();
}

export async function createLobby(game_id: string): Promise<Lobby> {
  const res = await fetch(`${API_BASE_URL}/lobbies`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ game_id }),
  });
  if (!res.ok) throw new Error("Failed to create lobby");
  return res.json();
}

import type { GameManifest } from './games';

export async function getLobby(id: string): Promise<Lobby & { manifest: GameManifest }> {
  const res = await fetch(`${API_BASE_URL}/lobbies/${id}`);
  if (!res.ok) throw new Error("Failed to fetch lobby");
  return res.json();
}