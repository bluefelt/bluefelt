import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';

export default function NotFoundPage() {
  const navigate = useNavigate();

  useEffect(() => {
    // Automatically redirect to home after a short delay
    const timeout = setTimeout(() => {
      navigate('/', { replace: true });
    }, 1500);

    return () => clearTimeout(timeout);
  }, [navigate]);

  return (
    <div className="min-h-screen flex items-center justify-center">
      <div className="text-center">
        <h1 className="text-6xl font-bold text-gray-400 mb-4">404</h1>
        <p className="text-xl text-gray-300 mb-4">Page not found</p>
        <p className="text-sm text-gray-500">Redirecting to home...</p>
      </div>
    </div>
  );
}