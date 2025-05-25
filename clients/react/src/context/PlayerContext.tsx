import React, { createContext, useContext, useState, useEffect, ReactNode } from "react";

type Player = {
  username: string;
};

type PlayerContextType = {
  player: Player | null;
  login: (username: string) => void;
  logout: () => void;
};

const PlayerContext = createContext<PlayerContextType | undefined>(undefined);

export const PlayerProvider = ({ children }: { children: ReactNode }) => {
  const [player, setPlayer] = useState<Player | null>(() => {
    const stored = localStorage.getItem("player");
    return stored ? JSON.parse(stored) : null;
  });

  useEffect(() => {
    if (player) localStorage.setItem("player", JSON.stringify(player));
    else localStorage.removeItem("player");
  }, [player]);

  const login = (username: string) => setPlayer({ username });
  const logout = () => setPlayer(null);

  return (
    <PlayerContext.Provider value={{ player, login, logout }}>
      {children}
    </PlayerContext.Provider>
  );
};

export function usePlayer() {
  const ctx = useContext(PlayerContext);
  if (!ctx) throw new Error("usePlayer must be used within a PlayerProvider");
  return ctx;
}