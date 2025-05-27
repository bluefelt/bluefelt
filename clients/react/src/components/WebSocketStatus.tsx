import type { WebSocketState } from '../ws/useReconnectingWebSocket';

interface Props {
  connected: boolean;
  state?: WebSocketState;
}

export default function WebSocketStatus({ connected, state }: Props) {
  const getStatusColor = () => {
    if (connected) return 'bg-green-500';
    if (state === 'connecting') return 'bg-yellow-500';
    if (state === 'error') return 'bg-red-500';
    return 'bg-gray-500';
  };

  const getStatusText = () => {
    if (connected) return 'Connected';
    if (state === 'connecting') return 'Connecting...';
    if (state === 'error') return 'Connection Error';
    return 'Disconnected';
  };

  return (
    <div className="fixed bottom-2 right-2 flex items-center space-x-2 text-sm bg-gray-800 bg-opacity-75 px-2 py-1 rounded">
      <span className={`w-2 h-2 rounded-full ${getStatusColor()}`} />
      <span>{getStatusText()}</span>
    </div>
  );
}
