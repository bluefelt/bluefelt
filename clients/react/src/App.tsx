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

export default function App() {
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