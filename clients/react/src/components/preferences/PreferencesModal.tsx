/**
 * Preferences Modal - Main UI for player customization
 */

import React, { useState } from 'react';
import { usePlayerPreferences } from '../../context/PlayerPreferencesContext';
import { useAnimation } from '../../context/AnimationContext';
import TokenSelector from './TokenSelector';
import ColorSchemeSelector from './ColorSchemeSelector';
import CardStyleSelector from './CardStyleSelector';
import PreviewPane from './PreviewPane';
import AnimationSettings from '../AnimationSettings';

interface PreferencesModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type TabId = 'appearance' | 'gameplay' | 'audio';

interface Tab {
  id: TabId;
  label: string;
  icon: string;
}

const TABS: Tab[] = [
  { id: 'appearance', label: 'Appearance', icon: '🎨' },
  { id: 'gameplay', label: 'Gameplay', icon: '🎮' },
  { id: 'audio', label: 'Audio', icon: '🔊' }
];

export const PreferencesModal: React.FC<PreferencesModalProps> = ({ isOpen, onClose }) => {
  const { preferences, updatePreferences, updateShowOpponentTokens, updatePlayerColor, updateCardStyle } = usePlayerPreferences();
  const { state: animationState, updateConfig } = useAnimation();
  const [activeTab, setActiveTab] = useState<TabId>('appearance');
  
  if (!isOpen || !preferences) return null;
  
  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-700">
          <h2 className="text-2xl font-bold text-white">Player Preferences</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-white transition-colors text-2xl leading-none"
            aria-label="Close preferences"
          >
            ×
          </button>
        </div>
        
        {/* Tabs */}
        <div className="flex border-b border-gray-700">
          {TABS.map(tab => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex items-center gap-2 px-6 py-3 font-medium transition-colors ${
                activeTab === tab.id
                  ? 'text-white border-b-2 border-blue-500'
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              <span>{tab.icon}</span>
              <span>{tab.label}</span>
            </button>
          ))}
        </div>
        
        {/* Content */}
        <div className="flex-1 overflow-hidden flex">
          {activeTab === 'appearance' && (
            <>
              {/* Left side - Options */}
              <div className="flex-1 p-6 overflow-y-auto">
                {/* Token Selection */}
                <div className="mb-8">
                  <h3 className="text-lg font-semibold text-white mb-4">Your Token</h3>
                  <TokenSelector
                    selectedTokenId={preferences.tokenId}
                    onTokenSelect={(tokenId) => updatePreferences({ tokenId })}
                  />
                </div>
                
                {/* Color Scheme Selection */}
                <div className="mb-8">
                  <h3 className="text-lg font-semibold text-white mb-4">Color Scheme</h3>
                  <ColorSchemeSelector
                    selectedSchemeId={preferences.colorSchemeId}
                    selectedPlayerColor={preferences.playerColor}
                    onSchemeSelect={(schemeId) => updatePreferences({ colorSchemeId: schemeId })}
                    onPlayerColorSelect={updatePlayerColor}
                  />
                </div>
                
                {/* Card Style Selection */}
                <div className="mb-8">
                  <h3 className="text-lg font-semibold text-white mb-4">Card Style</h3>
                  <CardStyleSelector
                    selectedStyleId={preferences.cardStyleId}
                    onStyleSelect={updateCardStyle}
                  />
                </div>
                
                {/* Opponent Token Display */}
                <div className="mb-8">
                  <h3 className="text-lg font-semibold text-white mb-4">Opponent Display</h3>
                  <div className="flex items-center justify-between p-4 bg-gray-700 rounded-lg">
                    <div>
                      <div className="text-white font-medium">Show Custom Opponent Tokens</div>
                      <div className="text-sm text-gray-400 mt-1">
                        See opponents' chosen tokens or use default tokens for everyone
                      </div>
                    </div>
                    <button
                      onClick={() => updateShowOpponentTokens(!preferences.showOpponentTokens)}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                        preferences.showOpponentTokens ? 'bg-blue-600' : 'bg-gray-600'
                      }`}
                      role="switch"
                      aria-checked={preferences.showOpponentTokens}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          preferences.showOpponentTokens ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>
                </div>
              </div>
              
              {/* Right side - Preview */}
              <div className="w-96 bg-gray-900 p-6 border-l border-gray-700">
                <h3 className="text-lg font-semibold text-white mb-4">Preview</h3>
                <PreviewPane
                  tokenId={preferences.tokenId}
                  colorSchemeId={preferences.colorSchemeId}
                  cardStyleId={preferences.cardStyleId}
                  showOpponentTokens={preferences.showOpponentTokens}
                  playerColor={preferences.playerColor}
                />
              </div>
            </>
          )}
          
          {activeTab === 'gameplay' && (
            <div className="flex-1 p-6">
              <div className="text-gray-400 text-center py-12">
                <p>Gameplay settings coming soon!</p>
                <p className="text-sm mt-2">Future options: Auto-pass, Confirm moves, etc.</p>
              </div>
            </div>
          )}
          
          {activeTab === 'audio' && (
            <div className="flex-1 p-6 overflow-y-auto">
              <div className="max-w-md">
                <h3 className="text-lg font-semibold text-white mb-6">Audio & Animation Settings</h3>
                
                {/* Animation Settings */}
                <div className="space-y-4">
                  {/* Enable Animations Toggle */}
                  <div className="flex items-center justify-between">
                    <label htmlFor="enable-animations" className="text-white font-medium">
                      Enable Animations
                    </label>
                    <button
                      id="enable-animations"
                      onClick={() => updateConfig({ enableAnimations: !animationState.config.enableAnimations })}
                      className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                        animationState.config.enableAnimations ? 'bg-blue-600' : 'bg-gray-600'
                      }`}
                      role="switch"
                      aria-checked={animationState.config.enableAnimations}
                    >
                      <span
                        className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                          animationState.config.enableAnimations ? 'translate-x-6' : 'translate-x-1'
                        }`}
                      />
                    </button>
                  </div>

                  {/* Animation Speed */}
                  <div className="space-y-2">
                    <label htmlFor="animation-speed" className="text-white font-medium block">
                      Animation Speed: {animationState.config.speed}x
                    </label>
                    <input
                      id="animation-speed"
                      type="range"
                      min="0.5"
                      max="3"
                      step="0.25"
                      value={animationState.config.speed}
                      onChange={(e) => updateConfig({ speed: parseFloat(e.target.value) })}
                      className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
                      disabled={!animationState.config.enableAnimations}
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
                      Pause Between Animations: {animationState.config.stillnessBetween}ms
                    </label>
                    <input
                      id="stillness-between"
                      type="range"
                      min="0"
                      max="1000"
                      step="50"
                      value={animationState.config.stillnessBetween}
                      onChange={(e) => updateConfig({ stillnessBetween: parseInt(e.target.value) })}
                      className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
                      disabled={!animationState.config.enableAnimations}
                    />
                    <div className="flex justify-between text-xs text-gray-400">
                      <span>0ms (None)</span>
                      <span>500ms</span>
                      <span>1000ms</span>
                    </div>
                  </div>

                  {/* Audio Settings Section */}
                  <div className="border-t border-gray-700 pt-4 mt-6">
                    <h4 className="text-white font-medium mb-3">Audio Settings</h4>
                    
                    {/* Master Audio Toggle */}
                    <div className="flex items-center justify-between mb-3">
                      <label htmlFor="enable-audio" className="text-white font-medium">
                        Sound Effects
                      </label>
                      <button
                        id="enable-audio"
                        onClick={() => updateConfig({ audioEnabled: !animationState.config.audioEnabled })}
                        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                          animationState.config.audioEnabled ? 'bg-blue-600' : 'bg-gray-600'
                        }`}
                        role="switch"
                        aria-checked={animationState.config.audioEnabled}
                      >
                        <span
                          className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                            animationState.config.audioEnabled ? 'translate-x-6' : 'translate-x-1'
                          }`}
                        />
                      </button>
                    </div>

                    {/* Volume Slider */}
                    {animationState.config.audioEnabled && (
                      <div className="space-y-2 mb-3">
                        <label htmlFor="audio-volume" className="text-white font-medium block">
                          Volume: {Math.round(animationState.config.audioVolume * 100)}%
                        </label>
                        <input
                          id="audio-volume"
                          type="range"
                          min="0"
                          max="1"
                          step="0.1"
                          value={animationState.config.audioVolume}
                          onChange={(e) => updateConfig({ audioVolume: parseFloat(e.target.value) })}
                          className="w-full h-2 bg-gray-600 rounded-lg appearance-none cursor-pointer slider"
                        />
                      </div>
                    )}

                    {/* Turn Notifications */}
                    {animationState.config.audioEnabled && (
                      <div className="flex items-center justify-between mb-3">
                        <label htmlFor="turn-notifications" className="text-white text-sm">
                          Turn Notifications
                        </label>
                        <button
                          id="turn-notifications"
                          onClick={() => updateConfig({ enableTurnNotifications: !animationState.config.enableTurnNotifications })}
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                            animationState.config.enableTurnNotifications ? 'bg-blue-600' : 'bg-gray-600'
                          }`}
                          role="switch"
                          aria-checked={animationState.config.enableTurnNotifications}
                        >
                          <span
                            className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                              animationState.config.enableTurnNotifications ? 'translate-x-5' : 'translate-x-1'
                            }`}
                          />
                        </button>
                      </div>
                    )}

                    {/* Action Sounds */}
                    {animationState.config.audioEnabled && (
                      <div className="flex items-center justify-between">
                        <label htmlFor="action-sounds" className="text-white text-sm">
                          Action Sounds
                        </label>
                        <button
                          id="action-sounds"
                          onClick={() => updateConfig({ enableActionSounds: !animationState.config.enableActionSounds })}
                          className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                            animationState.config.enableActionSounds ? 'bg-blue-600' : 'bg-gray-600'
                          }`}
                          role="switch"
                          aria-checked={animationState.config.enableActionSounds}
                        >
                          <span
                            className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                              animationState.config.enableActionSounds ? 'translate-x-5' : 'translate-x-1'
                            }`}
                          />
                        </button>
                      </div>
                    )}
                  </div>

                  {/* Reduce Motion */}
                  <div className="border-t border-gray-700 pt-4 mt-4">
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
                          reduceMotion: !animationState.config.reduceMotion,
                          enableAnimations: animationState.config.reduceMotion ? true : false
                        })}
                        className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 focus:ring-offset-gray-800 ${
                          animationState.config.reduceMotion ? 'bg-blue-600' : 'bg-gray-600'
                        }`}
                        role="switch"
                        aria-checked={animationState.config.reduceMotion}
                      >
                        <span
                          className={`inline-block h-4 w-4 transform rounded-full bg-white transition-transform ${
                            animationState.config.reduceMotion ? 'translate-x-6' : 'translate-x-1'
                          }`}
                        />
                      </button>
                    </div>
                  </div>

                  {/* Current Status */}
                  <div className="mt-6 p-3 bg-gray-700 rounded">
                    <h5 className="text-sm font-medium text-white mb-2">Current Status</h5>
                    <div className="text-xs text-gray-300 space-y-1">
                      <div>Animations: {animationState.config.enableAnimations && !animationState.config.reduceMotion ? 'Enabled' : 'Disabled'}</div>
                      <div>Audio: {animationState.config.audioEnabled ? 'Enabled' : 'Disabled'}</div>
                      <div>Queue: {animationState.queuedAnimations.length} animations</div>
                      <div>Active: {animationState.isAnimating ? 'Yes' : 'No'}</div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
        
        {/* Footer */}
        <div className="flex justify-end gap-3 p-6 border-t border-gray-700">
          <button
            onClick={onClose}
            className="px-6 py-2 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onClose}
            className="px-6 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
          >
            Save & Close
          </button>
        </div>
      </div>
    </div>
  );
};

export default PreferencesModal;