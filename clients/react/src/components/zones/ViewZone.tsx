import React, { useEffect, useState } from 'react';

interface ViewZoneData {
  players?: Record<string, Record<string, any>>;
  shared?: Record<string, any>;
  meta?: {
    labels?: Record<string, string>;
    lastUpdated?: number;
    updateFrequency?: string;
  };
}

interface ViewZoneProps {
  zoneId: string;
  zoneName: string;
  viewType: string;
  data: ViewZoneData;
  format?: {
    style: 'table' | 'list' | 'cards' | 'chart' | 'log' | 'summary';
    sortBy?: string;
    showDelta?: boolean;
    maxEntries?: number;
  };
  playerNames?: string[];
  you?: string;
  className?: string;
}

// Component to handle animated value updates
function AnimatedValue({ value, className = '' }: { value: any; className?: string }) {
  const [displayValue, setDisplayValue] = useState(value);
  const [isUpdating, setIsUpdating] = useState(false);
  
  useEffect(() => {
    if (value !== displayValue) {
      setIsUpdating(true);
      const timer = setTimeout(() => {
        setDisplayValue(value);
        setIsUpdating(false);
      }, 150);
      return () => clearTimeout(timer);
    }
  }, [value, displayValue]);
  
  return (
    <span 
      className={`inline-block transition-all duration-300 ${isUpdating ? 'scale-110' : 'scale-100'} ${className}`}
    >
      {displayValue}
    </span>
  );
}

export function ViewZone({
  zoneId,
  zoneName,
  viewType,
  data,
  format,
  playerNames = [],
  you,
  className = ''
}: ViewZoneProps) {
  const { players = {}, shared = {}, meta } = data;
  const labels = meta?.labels || {};
  const [isVisible, setIsVisible] = useState(false);
  
  useEffect(() => {
    // Trigger the fade-in animation
    const timer = setTimeout(() => setIsVisible(true), 50);
    return () => clearTimeout(timer);
  }, []);

  const renderTableView = () => {
    const allFields = new Set<string>();
    
    // Collect all field names
    Object.values(players).forEach(playerData => {
      Object.keys(playerData).forEach(field => allFields.add(field));
    });
    Object.keys(shared).forEach(field => allFields.add(field));
    
    const fields = Array.from(allFields);
    
    return (
      <div className="overflow-x-auto">
        <table className="w-full min-w-0">
          <thead>
            <tr className="border-b border-gray-600">
              <th className="text-left py-2 px-3 text-sm font-medium text-gray-300">Player</th>
              {fields.map(field => (
                <th key={field} className="text-center py-2 px-3 text-sm font-medium text-gray-300">
                  {labels[field] || field}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {Object.entries(players).map(([playerId, playerData], index) => {
              const playerIndex = parseInt(playerId.replace('p', '')) - 1;
              const playerName = playerNames[playerIndex] || playerId;
              const isYou = playerId === you;
              
              return (
                <tr
                  key={playerId}
                  className={`border-b border-gray-700 ${isYou ? 'bg-blue-900/20' : ''} 
                    animate-slide-in`}
                  style={{ animationDelay: `${index * 100}ms` }}
                >
                  <td className="py-2 px-3 text-sm font-medium">
                    {playerName}
                    {isYou && <span className="ml-2 text-xs text-blue-400">(You)</span>}
                  </td>
                  {fields.map(field => (
                    <td key={field} className="text-center py-2 px-3 text-sm">
                      <AnimatedValue value={playerData[field] ?? '-'} />
                    </td>
                  ))}
                </tr>
              );
            })}
            {Object.keys(shared).length > 0 && (
              <tr className="border-t-2 border-gray-600 animate-slide-in">
                <td className="py-2 px-3 text-sm font-medium text-gray-400">Total</td>
                {fields.map(field => (
                  <td key={field} className="text-center py-2 px-3 text-sm font-semibold">
                    <AnimatedValue 
                      value={shared[field] ?? '-'} 
                      className="text-yellow-400"
                    />
                  </td>
                ))}
              </tr>
            )}
          </tbody>
        </table>
      </div>
    );
  };

  const renderListView = () => {
    return (
      <div className="space-y-2">
        {Object.entries(players).map(([playerId, playerData], index) => {
          const playerIndex = parseInt(playerId.replace('p', '')) - 1;
          const playerName = playerNames[playerIndex] || playerId;
          const isYou = playerId === you;
          
          return (
            <div
              key={playerId}
              className={`p-3 rounded ${isYou ? 'bg-blue-900/20 border border-blue-700' : 'bg-gray-800/50'}
                animate-slide-in`}
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <div className="font-medium mb-1">
                {playerName}
                {isYou && <span className="ml-2 text-xs text-blue-400">(You)</span>}
              </div>
              <div className="grid grid-cols-2 gap-2 text-sm">
                {Object.entries(playerData).map(([field, value]) => (
                  <div key={field} className="flex justify-between">
                    <span className="text-gray-400">{labels[field] || field}:</span>
                    <span className="font-medium">
                      <AnimatedValue value={value} />
                    </span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
        
        {Object.keys(shared).length > 0 && (
          <div className="p-3 rounded bg-gray-700/50 border border-gray-600 animate-slide-in">
            <div className="font-medium mb-1 text-gray-300">Game Stats</div>
            <div className="grid grid-cols-2 gap-2 text-sm">
              {Object.entries(shared).map(([field, value]) => (
                <div key={field} className="flex justify-between">
                  <span className="text-gray-400">{labels[field] || field}:</span>
                  <span className="font-medium text-yellow-400">
                    <AnimatedValue value={value} />
                  </span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderCardsView = () => {
    return (
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
        {Object.entries(players).map(([playerId, playerData], index) => {
          const playerIndex = parseInt(playerId.replace('p', '')) - 1;
          const playerName = playerNames[playerIndex] || playerId;
          const isYou = playerId === you;
          
          return (
            <div
              key={playerId}
              className={`p-4 rounded-lg shadow-lg ${
                isYou ? 'bg-blue-900/30 border-2 border-blue-600' : 'bg-gray-800 border border-gray-700'
              } animate-scale-in hover:scale-105 transition-transform cursor-default`}
              style={{ animationDelay: `${index * 100}ms` }}
            >
              <h3 className="font-bold text-lg mb-3">
                {playerName}
                {isYou && <span className="ml-2 text-sm text-blue-400">(You)</span>}
              </h3>
              <div className="space-y-2">
                {Object.entries(playerData).map(([field, value]) => (
                  <div key={field} className="flex justify-between items-center">
                    <span className="text-sm text-gray-400">{labels[field] || field}</span>
                    <span className="text-xl font-bold">
                      <AnimatedValue value={value} />
                    </span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  const renderSummaryView = () => {
    return (
      <div className="flex flex-wrap gap-4 justify-center">
        {Object.entries(shared).map(([field, value], index) => (
          <div
            key={field}
            className="text-center animate-scale-in"
            style={{ animationDelay: `${index * 100}ms` }}
          >
            <div className="text-sm text-gray-400 mb-1">{labels[field] || field}</div>
            <div className="text-2xl font-bold text-yellow-400">
              <AnimatedValue value={value} />
            </div>
          </div>
        ))}
      </div>
    );
  };

  const renderContent = () => {
    const style = format?.style || 'table';
    
    switch (style) {
      case 'table':
        return renderTableView();
      case 'list':
        return renderListView();
      case 'cards':
        return renderCardsView();
      case 'summary':
        return renderSummaryView();
      default:
        return renderTableView();
    }
  };

  return (
    <div
      className={`view-zone view-zone-${viewType} ${className} transition-opacity duration-300 ${
        isVisible ? 'opacity-100' : 'opacity-0'
      }`}
      data-zone-id={zoneId}
      data-view-type={viewType}
    >
      <h2 className="text-lg font-semibold mb-3 text-gray-200">{zoneName}</h2>
      {renderContent()}
      
      <style dangerouslySetInnerHTML={{
        __html: `
        @keyframes slide-in {
          from {
            opacity: 0;
            transform: translateX(-10px);
          }
          to {
            opacity: 1;
            transform: translateX(0);
          }
        }
        
        @keyframes scale-in {
          from {
            opacity: 0;
            transform: scale(0.9);
          }
          to {
            opacity: 1;
            transform: scale(1);
          }
        }
        
        .animate-slide-in {
          animation: slide-in 0.3s ease-out forwards;
          opacity: 0;
        }
        
        .animate-scale-in {
          animation: scale-in 0.3s ease-out forwards;
          opacity: 0;
        }
        `
      }} />
    </div>
  );
}

export default ViewZone;