import { createContext, useContext, useState, useEffect, type ReactNode } from "react";
import type { ColorId } from "../config/colors";
import { DEFAULT_COLOR } from "../config/colors";

type Player = {
  username: string;
  color: ColorId;
};

type PlayerContextType = {
  player: Player | null;
  login: (username: string) => void;
  logout: () => void;
  updateColor: (color: ColorId) => void;
};

const PlayerContext = createContext<PlayerContextType | undefined>(undefined);

export const PlayerProvider = ({ children }: { children: ReactNode }) => {
  const [player, setPlayer] = useState<Player | null>(() => {
    const stored = localStorage.getItem("player");
    if (stored) {
      const parsed = JSON.parse(stored);
      // Ensure color is set (for backwards compatibility)
      if (!parsed.color) {
        parsed.color = DEFAULT_COLOR;
      }
      return parsed;
    }
    return null;
  });

  useEffect(() => {
    if (player) localStorage.setItem("player", JSON.stringify(player));
    else localStorage.removeItem("player");
  }, [player]);

  const login = (username: string) => setPlayer({ username, color: DEFAULT_COLOR });
  const logout = () => setPlayer(null);
  const updateColor = (color: ColorId) => {
    if (player) {
      setPlayer({ ...player, color });
    }
  };

  return (
    <PlayerContext.Provider value={{ player, login, logout, updateColor }}>
      {children}
    </PlayerContext.Provider>
  );
};

export function usePlayer() {
  const ctx = useContext(PlayerContext);
  if (!ctx) throw new Error("usePlayer must be used within a PlayerProvider");
  return ctx;
}