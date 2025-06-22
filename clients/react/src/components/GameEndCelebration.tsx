/**
 * GameEndCelebration Component
 * 
 * Provides celebration animations for game end scenarios:
 * - Victory celebration with confetti
 * - Defeat acknowledgment
 * - Tie game neutral animation
 */

import React, { useEffect, useState } from 'react';
import { useAnimation } from '../context/AnimationContext';

interface GameEndCelebrationProps {
  gameResult: 'victory' | 'defeat' | 'tie' | null;
  winner?: string;
  playerNames: string[];
  you: string;
  onComplete?: () => void;
  duration?: number;
  className?: string;
}

export const GameEndCelebration: React.FC<GameEndCelebrationProps> = ({
  gameResult,
  winner,
  playerNames,
  you,
  onComplete,
  duration = 3000,
  className = ''
}) => {
  const { state } = useAnimation();
  const { config } = state;
  const [isVisible, setIsVisible] = useState(false);
  const [animationPhase, setAnimationPhase] = useState<'enter' | 'celebrate' | 'exit'>('enter');

  // Get winner display name
  const getWinnerName = (): string => {
    if (!winner) return '';
    const playerIndex = parseInt(winner.replace('p', '')) - 1;
    return playerNames[playerIndex] || winner;
  };

  const winnerName = getWinnerName();
  const isYouWinner = winner === you;

  // Control celebration lifecycle
  useEffect(() => {
    if (gameResult && config.enableAnimations && !config.reduceMotion) {
      setIsVisible(true);
      setAnimationPhase('enter');

      // Enter phase
      const enterTimer = setTimeout(() => {
        setAnimationPhase('celebrate');
      }, 500 / config.speed);

      // Exit phase
      const exitTimer = setTimeout(() => {
        setAnimationPhase('exit');
      }, (duration - 500) / config.speed);

      // Complete
      const completeTimer = setTimeout(() => {
        setIsVisible(false);
        onComplete?.();
      }, duration / config.speed);

      return () => {
        clearTimeout(enterTimer);
        clearTimeout(exitTimer);
        clearTimeout(completeTimer);
      };
    } else if (gameResult) {
      // No animations - show briefly then complete
      setIsVisible(true);
      const timer = setTimeout(() => {
        setIsVisible(false);
        onComplete?.();
      }, 1000);
      return () => clearTimeout(timer);
    }
  }, [gameResult, config.enableAnimations, config.reduceMotion, config.speed, duration, onComplete]);

  if (!gameResult || !isVisible) {
    return null;
  }

  const getMessage = (): string => {
    switch (gameResult) {
      case 'victory':
        return isYouWinner ? 'You Won!' : 'Victory!';
      case 'defeat':
        return `${winnerName} Won`;
      case 'tie':
        return "It's a Tie!";
      default:
        return 'Game Over';
    }
  };

  const getEmoji = (): string => {
    switch (gameResult) {
      case 'victory':
        return '🎉';
      case 'defeat':
        return '👏';
      case 'tie':
        return '🤝';
      default:
        return '🎮';
    }
  };

  const getCelebrationClass = (): string => {
    if (!config.enableAnimations || config.reduceMotion) {
      return 'celebration-simple';
    }

    switch (gameResult) {
      case 'victory':
        return 'celebration-victory';
      case 'defeat':
        return 'celebration-defeat';
      case 'tie':
        return 'celebration-tie';
      default:
        return 'celebration-neutral';
    }
  };

  return (
    <div className={`fixed inset-0 z-50 flex items-center justify-center ${className}`}>
      {/* Background overlay */}
      <div 
        className={`absolute inset-0 bg-black transition-opacity duration-500 ${
          animationPhase === 'enter' ? 'bg-opacity-0' : 
          animationPhase === 'celebrate' ? 'bg-opacity-60' : 'bg-opacity-0'
        }`} 
      />

      {/* Confetti for victory */}
      {gameResult === 'victory' && config.enableAnimations && !config.reduceMotion && (
        <ConfettiEffect />
      )}

      {/* Main celebration content */}
      <div 
        className={`relative z-10 text-center ${getCelebrationClass()}`}
        style={{ 
          animationDuration: `${duration / config.speed}ms` 
        }}
      >
        <div className="bg-gray-800 bg-opacity-90 rounded-lg p-8 max-w-md mx-4 backdrop-blur-sm">
          {/* Emoji */}
          <div className="text-6xl mb-4 emoji-bounce">
            {getEmoji()}
          </div>

          {/* Main message */}
          <h2 className={`text-3xl font-bold mb-2 ${
            gameResult === 'victory' ? 'text-green-400' :
            gameResult === 'defeat' ? 'text-red-400' :
            'text-yellow-400'
          }`}>
            {getMessage()}
          </h2>

          {/* Additional info */}
          {gameResult === 'defeat' && winner && (
            <p className="text-gray-300 text-lg">
              Better luck next time!
            </p>
          )}

          {gameResult === 'tie' && (
            <p className="text-gray-300 text-lg">
              Great game everyone!
            </p>
          )}

          {/* Animated underline */}
          <div className="mt-4 mx-auto w-24 h-1 bg-gradient-to-r from-transparent via-current to-transparent opacity-75 animate-pulse" />
        </div>
      </div>

      <style jsx>{`
        .celebration-victory {
          animation: victoryBounce 0.8s ease-out;
        }
        
        .celebration-defeat {
          animation: defeatFade 0.6s ease-in-out;
        }
        
        .celebration-tie {
          animation: tieSway 0.7s ease-in-out;
        }
        
        .celebration-neutral {
          animation: neutralSlide 0.5s ease-out;
        }
        
        .celebration-simple {
          animation: simpleFade 0.3s ease-out;
        }
        
        .emoji-bounce {
          animation: emojiBounce 1s ease-out infinite;
        }
        
        @keyframes victoryBounce {
          0% {
            transform: scale(0.3) rotate(-10deg);
            opacity: 0;
          }
          50% {
            transform: scale(1.1) rotate(5deg);
            opacity: 1;
          }
          100% {
            transform: scale(1) rotate(0deg);
            opacity: 1;
          }
        }
        
        @keyframes defeatFade {
          0% {
            transform: scale(1.1);
            opacity: 0;
          }
          100% {
            transform: scale(1);
            opacity: 1;
          }
        }
        
        @keyframes tieSway {
          0%, 100% {
            transform: rotate(0deg);
            opacity: 0;
          }
          25% {
            transform: rotate(-2deg);
            opacity: 0.5;
          }
          75% {
            transform: rotate(2deg);
            opacity: 1;
          }
        }
        
        @keyframes neutralSlide {
          0% {
            transform: translateY(-20px);
            opacity: 0;
          }
          100% {
            transform: translateY(0);
            opacity: 1;
          }
        }
        
        @keyframes simpleFade {
          0% {
            opacity: 0;
          }
          100% {
            opacity: 1;
          }
        }
        
        @keyframes emojiBounce {
          0%, 100% {
            transform: translateY(0);
          }
          50% {
            transform: translateY(-10px);
          }
        }
      `}</style>
    </div>
  );
};

// Confetti effect component
const ConfettiEffect: React.FC = () => {
  const colors = ['#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7', '#dda0dd'];
  const particleCount = 50;

  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden">
      {Array.from({ length: particleCount }).map((_, i) => (
        <div
          key={i}
          className="absolute w-2 h-2 rounded-full confetti-particle"
          style={{
            backgroundColor: colors[Math.floor(Math.random() * colors.length)],
            left: `${Math.random() * 100}%`,
            top: '-10px',
            animationDelay: `${Math.random() * 2}s`,
            animationDuration: `${2 + Math.random() * 2}s`
          }}
        />
      ))}
      
      <style jsx>{`
        .confetti-particle {
          animation: confettiFall linear infinite;
        }
        
        @keyframes confettiFall {
          0% {
            transform: translateY(-10px) rotate(0deg);
            opacity: 1;
          }
          100% {
            transform: translateY(100vh) rotate(720deg);
            opacity: 0;
          }
        }
      `}</style>
    </div>
  );
};

export default GameEndCelebration;