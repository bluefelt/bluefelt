import { useQuery } from '@tanstack/react-query';

export function useLobbyUsers(id: string) {
  return useQuery({
    queryKey: ['lobbyUsers', id],
    queryFn: async () => {
      const res = await fetch(`http://localhost:8000/lobbies/${id}/users`);
      return (await res.json()) as string[];
    },
    refetchInterval: 2000,
  });
}
