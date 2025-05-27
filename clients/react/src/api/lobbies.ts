import { apiClient } from './client';
import type { GameManifest } from './games';

export type Lobby = {
  id: string;
  game_id: string;
  name: string;
  players: string[];
  started: boolean;
};

export async function getLobbies(): Promise<Lobby[]> {
  return apiClient.get<Lobby[]>('/lobbies');
}

export async function createLobby(game_id: string): Promise<Lobby> {
  return apiClient.post<Lobby>('/lobbies', { game_id });
}

export async function getLobby(id: string): Promise<Lobby & { manifest: GameManifest }> {
  return apiClient.get<Lobby & { manifest: GameManifest }>(`/lobbies/${id}`);
}