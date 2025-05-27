import { Navigate, useLocation } from 'react-router-dom';
import { usePlayer } from '../context/PlayerContext';

interface ProtectedRouteProps {
  children: React.ReactNode;
  redirectTo?: string;
}

export default function ProtectedRoute({ children, redirectTo = '/login' }: ProtectedRouteProps) {
  const { player } = usePlayer();
  const location = useLocation();

  if (!player) {
    // Preserve the attempted location for redirecting after login
    return <Navigate to={redirectTo} state={{ from: location }} replace />;
  }

  return <>{children}</>;
}