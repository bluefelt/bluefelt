import { createContext, useContext, type ReactNode } from "react";
import type { ColorId } from "../config/colors";
import { usePlayerPreferences } from "./PlayerPreferencesContext";

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

// This is now a wrapper around PlayerPreferencesProvider for backward compatibility
export const PlayerProvider = ({ children }: { children: ReactNode }) => {
  return <>{children}</>; // Just pass through - PlayerPreferencesProvider handles everything
};

export function usePlayer() {
  const { preferences, login: loginPref, logout: logoutPref, updateColor } = usePlayerPreferences();
  
  // Convert to old format
  const player = preferences ? {
    username: preferences.username,
    color: preferences.color
  } : null;
  
  return {
    player,
    login: loginPref,
    logout: logoutPref,
    updateColor
  };
}