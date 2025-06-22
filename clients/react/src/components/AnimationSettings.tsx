/**
 * Animation Settings Component
 * 
 * Provides user interface for configuring animation preferences.
 */

import React, { useState } from 'react';
import { useAnimation } from '../context/AnimationContext';

interface AnimationSettingsProps {
  isOpen: boolean;
  onClose: () => void;
}

export const AnimationSettings: React.FC<AnimationSettingsProps> = ({ isOpen, onClose }) => {
  const { state, updateConfig } = useAnimation();
  const { config } = state;

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 max-w-md w-full mx-4">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-xl font-bold text-white">Audio & Animation Settings</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors"
            aria-label="Close settings"
          >
            ✕
          </button>
        </div>

        <div className="space-y-4">
          {/* Enable Animations Toggle */}
          <div className="flex items-center justify-between">
            <label htmlFor="enable-animations" className="text-white font-medium">
              Enable Animations
            </label>
            <button
              id="enable-animations"
              onClick={() => updateConfig({ enableAnimations: !config.enableAnimations })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                config.enableAnimations ? 'bg-blue-600' : 'bg-gray-600'
              }`}
              role="switch"
              aria-checked={config.enableAnimations}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  config.enableAnimations ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>

          {/* Animation Speed */}
          <div className="space-y-2">
            <label htmlFor="animation-speed" className="text-white font-medium block">
              Animation Speed: {config.speed}x
            </label>
            <input
              id="animation-speed"
              type="range"
              min="0.5"
              max="3"
              step="0.25"
              value={config.speed}
              onChange={(e) => updateConfig({ speed: parseFloat(e.target.value) })}
              className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
              disabled={!config.enableAnimations}
            />
            <div className="flex justify-between text-xs text-gray-400">
              <span>0.5x (Slow)</span>
              <span>1x (Normal)</span>
              <span>3x (Fast)</span>
            </div>
          </div>

          {/* Stillness Between Animations */}
          <div className="space-y-2">
            <label htmlFor="stillness-between" className="text-white font-medium block">
              Pause Between Animations: {config.stillnessBetween}ms
            </label>
            <input
              id="stillness-between"
              type="range"
              min="0"
              max="1000"
              step="50"
              value={config.stillnessBetween}
              onChange={(e) => updateConfig({ stillnessBetween: parseInt(e.target.value) })}
              className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
              disabled={!config.enableAnimations}
            />
            <div className="flex justify-between text-xs text-gray-400">
              <span>0ms (None)</span>
              <span>500ms</span>
              <span>1000ms</span>
            </div>
          </div>

          {/* Audio Settings Section */}
          <div className="border-t border-gray-700 pt-4 mt-4">
            <h3 className="text-white font-medium mb-3">Audio Settings</h3>
            
            {/* Master Audio Toggle */}
            <div className="flex items-center justify-between mb-3">
              <label htmlFor="enable-audio" className="text-white font-medium">
                Sound Effects
              </label>
              <button
                id="enable-audio"
                onClick={() => updateConfig({ audioEnabled: !config.audioEnabled })}
                className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                  config.audioEnabled ? 'bg-blue-600' : 'bg-gray-600'
                }`}
                role="switch"
                aria-checked={config.audioEnabled}
              >
                <span
                  className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                    config.audioEnabled ? 'translate-x-6' : 'translate-x-1'
                  }`}
                />
              </button>
            </div>

            {/* Volume Slider */}
            {config.audioEnabled && (
              <div className="space-y-2 mb-3">
                <label htmlFor="audio-volume" className="text-white font-medium block">
                  Volume: {Math.round(config.audioVolume * 100)}%
                </label>
                <input
                  id="audio-volume"
                  type="range"
                  min="0"
                  max="1"
                  step="0.1"
                  value={config.audioVolume}
                  onChange={(e) => updateConfig({ audioVolume: parseFloat(e.target.value) })}
                  className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
                />
              </div>
            )}

            {/* Turn Notifications */}
            {config.audioEnabled && (
              <div className="flex items-center justify-between mb-3">
                <label htmlFor="turn-notifications" className="text-white text-sm">
                  Turn Notifications
                </label>
                <button
                  id="turn-notifications"
                  onClick={() => updateConfig({ enableTurnNotifications: !config.enableTurnNotifications })}
                  className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                    config.enableTurnNotifications ? 'bg-blue-600' : 'bg-gray-600'
                  }`}
                  role="switch"
                  aria-checked={config.enableTurnNotifications}
                >
                  <span
                    className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                      config.enableTurnNotifications ? 'translate-x-5' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>
            )}

            {/* Action Sounds */}
            {config.audioEnabled && (
              <div className="flex items-center justify-between">
                <label htmlFor="action-sounds" className="text-white text-sm">
                  Action Sounds
                </label>
                <button
                  id="action-sounds"
                  onClick={() => updateConfig({ enableActionSounds: !config.enableActionSounds })}
                  className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                    config.enableActionSounds ? 'bg-blue-600' : 'bg-gray-600'
                  }`}
                  role="switch"
                  aria-checked={config.enableActionSounds}
                >
                  <span
                    className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                      config.enableActionSounds ? 'translate-x-5' : 'translate-x-1'
                    }`}
                  />
                </button>
              </div>
            )}
          </div>

          {/* Reduce Motion */}
          <div className="flex items-center justify-between">
            <label htmlFor="reduce-motion" className="text-white font-medium">
              Reduce Motion
              <span className="block text-xs text-gray-400 font-normal">
                For accessibility or motion sensitivity
              </span>
            </label>
            <button
              id="reduce-motion"
              onClick={() => updateConfig({ 
                reduceMotion: !config.reduceMotion,
                enableAnimations: config.reduceMotion ? true : false // Auto-disable animations when reducing motion
              })}
              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                config.reduceMotion ? 'bg-blue-600' : 'bg-gray-600'
              }`}
              role="switch"
              aria-checked={config.reduceMotion}
            >
              <span
                className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                  config.reduceMotion ? 'translate-x-6' : 'translate-x-1'
                }`}
              />
            </button>
          </div>
        </div>

        {/* Current Animation Status */}
        <div className="mt-6 p-3 bg-gray-700 rounded">
          <h3 className="text-sm font-medium text-white mb-2">Current Status</h3>
          <div className="text-xs text-gray-300 space-y-1">
            <div>Animations: {config.enableAnimations && !config.reduceMotion ? 'Enabled' : 'Disabled'}</div>
            <div>Audio: {config.audioEnabled ? 'Enabled' : 'Disabled'}</div>
            <div>Queue: {state.queuedAnimations.length} animations</div>
            <div>Active: {state.isAnimating ? 'Yes' : 'No'}</div>
          </div>
        </div>

        {/* Close Button */}
        <div className="mt-6 flex justify-end">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  );
};

export default AnimationSettings;