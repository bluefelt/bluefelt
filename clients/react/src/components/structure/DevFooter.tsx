import { Link } from 'react-router-dom';
import { useLiveReload } from '../../hooks/useLiveReload.tsx';

export default function DevFooter() {
  const { connected, lastReload } = useLiveReload();

  // Only render in development
  if (!import.meta.env.DEV) {
    return null;
  }

  return (
    <footer className="bg-gray-800 border-t border-gray-700 mt-auto">
      <div className="max-w-7xl mx-auto px-4 py-3">
        <div className="flex items-center justify-between text-sm text-gray-400">
          <div className="flex items-center gap-6">
            <span className="font-medium text-yellow-400">DEV MODE</span>
            <Link 
              to="/ui-test" 
              className="hover:text-white transition-colors underline"
            >
              UI Test Harness
            </Link>
          </div>
          
          <div className="flex items-center gap-6 text-xs">
            <div>
              Server: {import.meta.env.VITE_API_URL || 'http://localhost:8000'}
            </div>
            
            <div className="flex items-center gap-2">
              <div className={`w-2 h-2 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
              <span className="font-mono">
                {connected ? 'Live Reload' : 'Offline'}
              </span>
              {lastReload && (
                <span className="text-gray-500">
                  ({lastReload.toLocaleTimeString()})
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
    </footer>
  );
}