import { Link } from 'react-router-dom';

export default function DevFooter() {
  // Only render in development
  if (!import.meta.env.DEV) {
    return null;
  }

  return (
    <footer className="bg-gray-800 border-t border-gray-700 mt-auto">
      <div className="max-w-7xl mx-auto px-4 py-3">
        <div className="flex items-center justify-between text-sm text-gray-400">
          <div className="flex items-center gap-4">
            <span className="font-medium text-yellow-400">DEV MODE</span>
            <Link 
              to="/ui-test" 
              className="hover:text-white transition-colors underline"
            >
              UI Test Harness
            </Link>
          </div>
          <div className="text-xs">
            Server: {import.meta.env.VITE_API_URL || 'http://localhost:8000'}
          </div>
        </div>
      </div>
    </footer>
  );
}