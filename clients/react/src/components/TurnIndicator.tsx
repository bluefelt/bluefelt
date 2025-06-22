/**
 * TurnIndicator Component
 * 
 * Provides visual feedback for turn changes with animated effects.
 * Shows whose turn it is and animates when turns change.
 */

import React, { useEffect, useState } from 'react';
import { useAnimation } from '../context/AnimationContext';

interface TurnIndicatorProps {
  currentPlayer: string;
  previousPlayer?: string;
  playerNames: string[];
  you: string;
  isTransitioning?: boolean;
  className?: string;
}

export const TurnIndicator: React.FC<TurnIndicatorProps> = ({
  currentPlayer,
  previousPlayer,
  playerNames,
  you,
  isTransitioning = false,
  className = ''
}) => {
  const { state } = useAnimation();
  const { config } = state;
  const [isAnimating, setIsAnimating] = useState(false);

  // Get player display name
  const getPlayerName = (playerId: string): string => {
    const playerIndex = parseInt(playerId.replace('p', '')) - 1;
    return playerNames[playerIndex] || playerId;
  };

  const currentPlayerName = getPlayerName(currentPlayer);
  const isYourTurn = currentPlayer === you;

  // Trigger animation when turn changes
  useEffect(() => {
    if (previousPlayer && previousPlayer !== currentPlayer && config.enableAnimations && !config.reduceMotion) {
      setIsAnimating(true);
      
      const timer = setTimeout(() => {
        setIsAnimating(false);
      }, 800 / config.speed);

      return () => clearTimeout(timer);
    }
  }, [currentPlayer, previousPlayer, config.enableAnimations, config.reduceMotion, config.speed]);

  const baseClasses = `turn-indicator inline-flex items-center space-x-3 px-4 py-2 rounded-lg transition-all duration-300 ${className}`;
  
  const stateClasses = isYourTurn
    ? 'bg-green-900 border-2 border-green-600 text-green-100'
    : 'bg-gray-800 border-2 border-gray-600 text-gray-300';

  const animationClasses = isAnimating
    ? 'animate-pulse scale-105 ring-4 ring-blue-500 ring-opacity-50'
    : '';

  return (
    <div className={`${baseClasses} ${stateClasses} ${animationClasses}`}>
      {/* Turn status icon */}
      <div className="relative">
        {isYourTurn ? (
          <div className="w-3 h-3 bg-green-500 rounded-full animate-pulse" />
        ) : (
          <div className="w-3 h-3 bg-gray-500 rounded-full" />
        )}
        
        {/* Animated ring for active turn */}
        {isAnimating && (
          <div className="absolute inset-0 w-3 h-3 bg-blue-500 rounded-full animate-ping opacity-75" />
        )}
      </div>

      {/* Player name */}
      <div className="flex flex-col">
        <span className="text-sm font-medium">
          {isYourTurn ? 'Your Turn' : `${currentPlayerName}'s Turn`}
        </span>
        
        {/* Turn change message during transition */}
        {(isAnimating || isTransitioning) && (
          <span className="text-xs opacity-75 animate-fade-in">
            Turn changed
          </span>
        )}
      </div>

      {/* Turn timer visualization (optional enhancement) */}
      {isYourTurn && (
        <div className="ml-auto">
          <div className="w-6 h-6 rounded-full border-2 border-green-500 relative">
            <div className="absolute inset-1 bg-green-500 rounded-full opacity-20 animate-pulse" />
          </div>
        </div>
      )}

      <style dangerouslySetInnerHTML={{
        __html: `
        @keyframes fade-in {
          from {
            opacity: 0;
            transform: translateY(-2px);
          }
          to {
            opacity: 0.75;
            transform: translateY(0);
          }
        }
        
        .animate-fade-in {
          animation: fade-in 0.3s ease-out forwards;
        }
        
        /* Accessibility: Reduce motion when requested */
        @media (prefers-reduced-motion: reduce) {
          .turn-indicator {
            animation: none !important;
          }
          
          .animate-pulse {
            animation: none !important;
          }
          
          .animate-ping {
            animation: none !important;
          }
        }
        `
      }} />
    </div>
  );
};

export default TurnIndicator;