import { useEffect, useCallback, useState } from 'react';

/**
 * Hook to connect to the server's live reload WebSocket in development
 * Automatically refreshes the page when game bundles are reloaded
 */
export function useLiveReload() {
  const [connected, setConnected] = useState(false);
  const [lastReload, setLastReload] = useState<Date | null>(null);

  useEffect(() => {
    // Only enable in development
    if (import.meta.env.PROD) {
      return;
    }

    let ws: WebSocket | null = null;
    let reconnectTimeout: NodeJS.Timeout | null = null;

    const connect = () => {
      try {
        ws = new WebSocket('ws://localhost:8000/api/reload/ws');

        ws.onopen = () => {
          console.log('[LiveReload] Connected');
          setConnected(true);
        };

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data);
            
            switch (data.type) {
              case 'connected':
                console.log('[LiveReload]', data.message);
                break;
                
              case 'gameReloaded':
                console.log(`[LiveReload] Game ${data.gameId} reloaded`);
                setLastReload(new Date());
                // Reload the page after a short delay
                setTimeout(() => {
                  window.location.reload();
                }, 100);
                break;
                
              case 'allGamesReloaded':
                console.log('[LiveReload] All games reloaded');
                setLastReload(new Date());
                // Reload the page after a short delay
                setTimeout(() => {
                  window.location.reload();
                }, 100);
                break;
            }
          } catch (e) {
            console.error('[LiveReload] Failed to parse message:', e);
          }
        };

        ws.onerror = (error) => {
          console.error('[LiveReload] WebSocket error:', error);
        };

        ws.onclose = () => {
          console.log('[LiveReload] Disconnected');
          setConnected(false);
          
          // Attempt to reconnect after 5 seconds
          reconnectTimeout = setTimeout(() => {
            console.log('[LiveReload] Attempting to reconnect...');
            connect();
          }, 5000);
        };

      } catch (error) {
        console.error('[LiveReload] Failed to connect:', error);
      }
    };

    // Initial connection
    connect();

    // Cleanup on unmount
    return () => {
      if (reconnectTimeout) {
        clearTimeout(reconnectTimeout);
      }
      if (ws) {
        ws.close();
      }
    };
  }, []);

  const forceReload = useCallback(() => {
    window.location.reload();
  }, []);

  return {
    connected,
    lastReload,
    forceReload,
  };
}

/**
 * Component to show live reload status in development
 */
export function LiveReloadIndicator() {
  const { connected, lastReload } = useLiveReload();

  // Only show in development
  if (import.meta.env.PROD) {
    return null;
  }

  return (
    <div
      className={`fixed bottom-4 right-4 px-3 py-1 rounded-full text-xs font-mono ${
        connected ? 'bg-green-500' : 'bg-red-500'
      } text-white shadow-lg z-50`}
    >
      {connected ? '🔄 Live' : '❌ Offline'}
      {lastReload && (
        <span className="ml-2 opacity-75">
          {lastReload.toLocaleTimeString()}
        </span>
      )}
    </div>
  );
}