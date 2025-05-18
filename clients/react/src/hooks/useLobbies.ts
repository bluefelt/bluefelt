import { useQuery } from '@tanstack/react-query';

export function useLobbies() {
  return useQuery({
    queryKey: ['lobbies'],
    queryFn: async () => {
      const res = await fetch('http://localhost:8000/lobbies');
      return (await res.json()) as { id: string; game_id: string }[];
    },
    refetchInterval: 2000
  });
}
