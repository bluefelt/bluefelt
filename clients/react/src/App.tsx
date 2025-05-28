import { BrowserRouter, Routes, Route } from "react-router-dom";
import { PlayerProvider } from "./context/PlayerContext";
import { WebSocketProvider } from "./context/WebSocketContext";
import Layout from "./components/Layout";
import HomePage from "./pages/HomePage";
import LoginPage from "./pages/LoginPage";
import LobbiesPage from "./pages/LobbiesPage";
import CreateLobbyPage from "./pages/CreateLobbyPage";
import LobbyPage from "./pages/LobbyPage";
import NotFoundPage from "./pages/NotFoundPage";
import "./index.css";
import { useEffect } from "react";

// Clear old lobby ticks on app start in development
function clearOldLobbyTicks() {
  if (import.meta.env.DEV) {
    const keys = Object.keys(localStorage);
    keys.forEach(key => {
      if (key.startsWith('lobby_') && key.endsWith('_lastTick')) {
        localStorage.removeItem(key);
      }
    });
  }
}

export default function App() {
  useEffect(() => {
    // Clear old lobby ticks on app start in development
    clearOldLobbyTicks();
  }, []);
  
  return (
    <BrowserRouter>
      <PlayerProvider>
        <WebSocketProvider>
          <Routes>
            <Route path="/" element={<Layout />}>
              <Route index element={<HomePage />} />
              <Route path="login" element={<LoginPage />} />
              <Route path="lobbies" element={<LobbiesPage />} />
              <Route path="create-lobby" element={<CreateLobbyPage />} />
              <Route path="lobby/:lobbyId" element={<LobbyPage />} />
              <Route path="*" element={<NotFoundPage />} />
            </Route>
          </Routes>
        </WebSocketProvider>
      </PlayerProvider>
    </BrowserRouter>
  );
}