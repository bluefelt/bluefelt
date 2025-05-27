import { apiClient } from './client';

export type Game = {
  id: string;
  name: string;
};

export async function getGames(): Promise<Game[]> {
  return apiClient.get<Game[]>('/games');
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
  return apiClient.get<GameManifest>(`/games/${gameId}`);
}