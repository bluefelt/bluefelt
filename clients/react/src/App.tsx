import { lazy, Suspense, useEffect } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { PlayerProvider } from "./context/PlayerContext";
import { WebSocketProvider } from "./context/WebSocketContext";
import Layout from "./layout/Layout.tsx";
import "./index.css";

// Lazy load page components
const HomePage = lazy(() => import("./pages/HomePage"));
const LoginPage = lazy(() => import("./pages/LoginPage"));
const LobbiesPage = lazy(() => import("./pages/LobbiesPage"));
const CreateLobbyPage = lazy(() => import("./pages/CreateLobbyPage"));
const LobbyPage = lazy(() => import("./pages/LobbyPage"));
const NotFoundPage = lazy(() => import("./pages/NotFoundPage"));

// Development-only pages
const UITestPage = import.meta.env.DEV 
  ? lazy(() => import("./pages/UITestPage"))
  : null;

// Loading component
function PageLoader() {
  return (
    <div className="flex items-center justify-center min-h-screen">
      <div className="text-center">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white mx-auto mb-4"></div>
        <p className="text-gray-300">Loading...</p>
      </div>
    </div>
  );
}

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
          <Suspense fallback={<PageLoader />}>
            <Routes>
              <Route path="/" element={<Layout />}>
                <Route index element={<HomePage />} />
                <Route path="login" element={<LoginPage />} />
                <Route path="lobbies" element={<LobbiesPage />} />
                <Route path="create-lobby" element={<CreateLobbyPage />} />
                <Route path="lobby/:lobbyId" element={<LobbyPage />} />
                {import.meta.env.DEV && UITestPage && (
                  <Route path="ui-test" element={<UITestPage />} />
                )}
                <Route path="*" element={<NotFoundPage />} />
              </Route>
            </Routes>
          </Suspense>
        </WebSocketProvider>
      </PlayerProvider>
    </BrowserRouter>
  );
}