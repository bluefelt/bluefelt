export type Game = {
  id: string;
  name: string;
};

export async function getGames(): Promise<Game[]> {
  const res = await fetch("/games");
  if (!res.ok) throw new Error("Failed to fetch list of games");
  return res.json();
}
