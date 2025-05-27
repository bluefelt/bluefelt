import { API_BASE_URL } from '../config';

export type Game = {
  id: string;
  name: string;
};

export async function getGames(): Promise<Game[]> {
  const res = await fetch(`${API_BASE_URL}/games`);
  if (!res.ok) throw new Error("Failed to fetch list of games");
  return res.json();
}

export type GameManifest = {
  gameId: string;
  version: string;
  specVersion: string;
  metadata: {
    name: string;
    author: string;
    players: { min: number; max: number };
    description: string;
  };
};

export async function getGameManifest(gameId: string): Promise<GameManifest> {
  const res = await fetch(`${API_BASE_URL}/games/${gameId}`);
  if (!res.ok) throw new Error("Failed to fetch game manifest");
  return res.json();
}