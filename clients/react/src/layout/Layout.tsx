import { Outlet } from 'react-router-dom';
import SiteHeader from '../components/structure/SiteHeader.tsx';

export default function Layout() {
  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <SiteHeader />
      <Outlet />
    </div>
  );
}