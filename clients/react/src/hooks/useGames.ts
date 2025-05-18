import { useQuery } from '@tanstack/react-query';

export function useGames() {
  return useQuery({
    queryKey: ['games'],
    queryFn: async () => {
      const res = await fetch('http://localhost:8000/games');
      return (await res.json()) as { id: string; name: string }[];
    },
  });
}
