import { BrowserRouter, Routes, Route } from "react-router-dom";
import { PlayerProvider } from "./context/PlayerContext.tsx";
import { WebSocketProvider } from "./context/WebSocketContext.tsx";
import Layout from "./components/Layout.tsx";
import HomePage from "./pages/HomePage.tsx";
import LoginPage from "./pages/LoginPage.tsx";
import LobbiesPage from "./pages/LobbiesPage.tsx";
import CreateLobbyPage from "./pages/CreateLobbyPage.tsx";
import LobbyPage from "./pages/LobbyPage.tsx";
import NotFoundPage from "./pages/NotFoundPage.tsx";
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