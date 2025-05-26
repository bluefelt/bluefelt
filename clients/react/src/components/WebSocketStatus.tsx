import React from 'react';

export default function WebSocketStatus({ connected }: { connected: boolean }) {
  return (
    <div className="fixed bottom-2 right-2 flex items-center space-x-2 text-sm bg-gray-800 bg-opacity-75 px-2 py-1 rounded">
      <span
        className={
          'w-2 h-2 rounded-full ' + (connected ? 'bg-green-500' : 'bg-red-500')
        }
      />
      <span>{connected ? 'connected' : 'disconnected'}</span>
    </div>
  );
}
