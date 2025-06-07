import { Outlet } from 'react-router-dom';
import SiteHeader from '../components/structure/SiteHeader.tsx';
import DevFooter from '../components/structure/DevFooter.tsx';

export default function Layout() {
  return (
    <div className="min-h-screen bg-gray-900 text-white flex flex-col">
      <SiteHeader />
      <main className="flex-1">
        <Outlet />
      </main>
      <DevFooter />
    </div>
  );
}