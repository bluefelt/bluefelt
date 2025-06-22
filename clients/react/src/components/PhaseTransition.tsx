/**
 * PhaseTransition Component
 * 
 * Provides visual feedback for game phase transitions with animated effects.
 * Integrates with the animation system to show smooth transitions between phases.
 */

import React, { useEffect, useState } from 'react';
import { useAnimation } from '../context/AnimationContext';

interface PhaseTransitionProps {
  currentPhase: string;
  previousPhase?: string;
  isTransitioning?: boolean;
  transitionMessage?: string;
  duration?: number;
  className?: string;
}

export const PhaseTransition: React.FC<PhaseTransitionProps> = ({
  currentPhase,
  previousPhase,
  isTransitioning = false,
  transitionMessage,
  duration = 1000,
  className = ''
}) => {
  const { state } = useAnimation();
  const { config } = state;
  const [showTransition, setShowTransition] = useState(false);
  const [displayPhase, setDisplayPhase] = useState(currentPhase);

  // Trigger transition when phase changes
  useEffect(() => {
    if (previousPhase && previousPhase !== currentPhase && config.enableAnimations && !config.reduceMotion) {
      setShowTransition(true);
      
      // Update display phase after animation
      const timer = setTimeout(() => {
        setDisplayPhase(currentPhase);
        setShowTransition(false);
      }, duration / config.speed);

      return () => clearTimeout(timer);
    } else {
      setDisplayPhase(currentPhase);
    }
  }, [currentPhase, previousPhase, config.enableAnimations, config.reduceMotion, config.speed, duration]);

  // Format phase name for display
  const formatPhaseName = (phase: string): string => {
    return phase
      .replace(/([A-Z])/g, ' $1')
      .replace(/^./, str => str.toUpperCase())
      .trim();
  };

  const formattedCurrentPhase = formatPhaseName(displayPhase);
  const formattedPreviousPhase = previousPhase ? formatPhaseName(previousPhase) : '';

  // Render transition overlay when actively transitioning
  if (showTransition || isTransitioning) {
    return (
      <div 
        className={`fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 transition-opacity duration-300 ${className}`}
        style={{ 
          animationDuration: `${duration / config.speed}ms` 
        }}
      >
        <div className="bg-gray-800 rounded-lg p-8 max-w-md mx-4 text-center">
          {/* Phase transition animation */}
          <div className="space-y-4">
            {/* Previous phase fade out */}
            {formattedPreviousPhase && (
              <div className="phase-exit">
                <div className="text-lg font-medium text-gray-400 opacity-60">
                  {formattedPreviousPhase}
                </div>
                <div className="w-full h-1 bg-gray-600 rounded mt-2">
                  <div className="h-1 bg-gray-400 rounded animate-pulse" style={{ width: '100%' }} />
                </div>
              </div>
            )}

            {/* Transition message or arrow */}
            <div className="transition-indicator flex justify-center items-center py-2">
              {transitionMessage ? (
                <div className="text-sm text-gray-300 animate-pulse">
                  {transitionMessage}
                </div>
              ) : (
                <div className="flex items-center space-x-2">
                  <div className="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: '0ms' }} />
                  <div className="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: '150ms' }} />
                  <div className="w-2 h-2 bg-blue-500 rounded-full animate-bounce" style={{ animationDelay: '300ms' }} />
                </div>
              )}
            </div>

            {/* New phase fade in */}
            <div className="phase-enter">
              <div className="text-xl font-bold text-white">
                {formattedCurrentPhase}
              </div>
              <div className="w-full h-2 bg-gray-600 rounded mt-2">
                <div className="h-2 bg-blue-500 rounded animate-pulse" style={{ width: '0%' }} />
              </div>
            </div>
          </div>
        </div>

        <style dangerouslySetInnerHTML={{
          __html: `
          .phase-exit {
            animation: slideOutUp 0.3s ease-in forwards;
          }
          
          .phase-enter {
            animation: slideInDown 0.3s ease-out 0.2s both;
          }
          
          .phase-enter .h-2 {
            animation: fillBar 0.5s ease-out 0.4s forwards;
          }
          
          @keyframes slideOutUp {
            from {
              opacity: 1;
              transform: translateY(0);
            }
            to {
              opacity: 0;
              transform: translateY(-20px);
            }
          }
          
          @keyframes slideInDown {
            from {
              opacity: 0;
              transform: translateY(-20px);
            }
            to {
              opacity: 1;
              transform: translateY(0);
            }
          }
          
          @keyframes fillBar {
            from {
              width: 0%;
            }
            to {
              width: 100%;
            }
          }
          `
        }} />
      </div>
    );
  }

  // Render subtle phase indicator when not transitioning
  return (
    <div className={`phase-indicator ${className}`}>
      <div className="inline-flex items-center space-x-2 px-3 py-1 bg-gray-800 rounded-full text-sm">
        <div className="w-2 h-2 bg-blue-500 rounded-full" />
        <span className="text-gray-300">{formattedCurrentPhase}</span>
      </div>
    </div>
  );
};

export default PhaseTransition;