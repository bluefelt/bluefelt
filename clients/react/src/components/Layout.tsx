import { Outlet } from 'react-router-dom';
import PlayerProfile from './PlayerProfile.tsx';

export default function Layout() {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <PlayerProfile />
      <Outlet />
    </div>
  );
}