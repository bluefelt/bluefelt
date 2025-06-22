/**
 * Color Scheme Selector Component
 */

import React, { useState } from 'react';
import { COLOR_SCHEMES, COLOR_OPTIONS, type ColorScheme, type ColorOption } from '../../tokens/ColorSchemes';

interface ColorSchemeSelectorProps {
  selectedSchemeId: string;
  selectedPlayerColor?: string;
  onSchemeSelect: (schemeId: string) => void;
  onPlayerColorSelect?: (color: string) => void;
}

const ColorSchemeSelector: React.FC<ColorSchemeSelectorProps> = ({
  selectedSchemeId,
  selectedPlayerColor,
  onSchemeSelect,
  onPlayerColorSelect
}) => {
  const [showColorPicker, setShowColorPicker] = useState(false);
  const selectedScheme = COLOR_SCHEMES.find(s => s.id === selectedSchemeId);
  
  // Get the actual player color (custom or default from scheme)
  const getPlayerColor = (scheme: ColorScheme) => {
    if (scheme.id === selectedSchemeId && selectedPlayerColor) {
      return selectedPlayerColor;
    }
    return scheme.playerColor;
  };

  return (
    <div className="space-y-4">
      {/* Scheme Selection */}
      <div className="grid grid-cols-2 gap-3">
        {COLOR_SCHEMES.map(scheme => (
          <button
            key={scheme.id}
            onClick={() => onSchemeSelect(scheme.id)}
            className={`p-4 rounded-lg border-2 transition-all ${
              selectedSchemeId === scheme.id
                ? 'border-blue-500 bg-blue-500 bg-opacity-20'
                : 'border-gray-600 hover:border-gray-500 bg-gray-700 hover:bg-gray-600'
            }`}
          >
            <div className="flex items-center justify-between mb-2">
              <span className="font-medium text-white">{scheme.name}</span>
              {selectedSchemeId === scheme.id && (
                <svg className="w-4 h-4 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                </svg>
              )}
            </div>
            <div className="text-xs text-gray-400 mb-3">{scheme.description}</div>
            
            {/* Color preview */}
            <div className="flex items-center gap-2">
              {/* Player color */}
              <div className="relative">
                <div
                  className="w-8 h-8 rounded-full border-2 border-white"
                  style={{ backgroundColor: getPlayerColor(scheme) }}
                  title="Your color"
                />
                <div className="absolute -bottom-1 -right-1 bg-gray-800 rounded-full p-0.5">
                  <span className="text-xs">👤</span>
                </div>
              </div>
              
              {/* Separator */}
              <div className="text-gray-500 text-xs">vs</div>
              
              {/* Opponent colors */}
              <div className="flex -space-x-2">
                {scheme.opponentColors.slice(0, 3).map((color, idx) => (
                  <div
                    key={idx}
                    className="w-6 h-6 rounded-full border border-gray-700"
                    style={{ 
                      backgroundColor: color,
                      zIndex: 3 - idx
                    }}
                    title={`Opponent ${idx + 1} color`}
                  />
                ))}
                {scheme.opponentColors.length > 3 && (
                  <div className="w-6 h-6 rounded-full bg-gray-600 border border-gray-700 flex items-center justify-center">
                    <span className="text-xs text-gray-300">+{scheme.opponentColors.length - 3}</span>
                  </div>
                )}
              </div>
            </div>
          </button>
        ))}
      </div>
      
      {/* Custom Color Selection for Selected Scheme */}
      {selectedScheme && onPlayerColorSelect && (
        <div className="border-t border-gray-600 pt-4">
          <div className="flex items-center justify-between mb-3">
            <h4 className="text-white font-medium">Customize Your Color</h4>
            <button
              onClick={() => setShowColorPicker(!showColorPicker)}
              className="text-sm px-3 py-1 bg-gray-700 text-white rounded hover:bg-gray-600 transition-colors"
            >
              {showColorPicker ? 'Hide Colors' : 'Pick Color'}
            </button>
          </div>
          
          {showColorPicker && (
            <div className="grid grid-cols-8 gap-2 p-3 bg-gray-800 rounded">
              {COLOR_OPTIONS.map(color => (
                <button
                  key={color.id}
                  onClick={() => onPlayerColorSelect(color.hex)}
                  className={`w-8 h-8 rounded-full border-2 transition-all hover:scale-110 ${
                    getPlayerColor(selectedScheme) === color.hex
                      ? 'border-white ring-2 ring-blue-400'
                      : 'border-gray-600 hover:border-gray-400'
                  }`}
                  style={{ backgroundColor: color.hex }}
                  title={color.name}
                />
              ))}
            </div>
          )}
          
          {/* Reset to default button */}
          {selectedPlayerColor && selectedPlayerColor !== selectedScheme.playerColor && (
            <button
              onClick={() => onPlayerColorSelect(selectedScheme.playerColor)}
              className="mt-2 text-sm text-gray-400 hover:text-white transition-colors"
            >
              Reset to default ({selectedScheme.name} scheme color)
            </button>
          )}
        </div>
      )}
    </div>
  );
};

export default ColorSchemeSelector;