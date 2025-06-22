/**
 * Card Style Selector Component
 * 
 * Allows users to choose from different card styles with live preview
 */

import React, { useState, useEffect } from 'react';
import { CardStyleRegistry, type CardStyle } from '../../cards/CardStyleRegistry';

interface CardStyleSelectorProps {
  selectedStyleId?: string;
  onStyleSelect: (styleId: string) => void;
}

export const CardStyleSelector: React.FC<CardStyleSelectorProps> = ({
  selectedStyleId,
  onStyleSelect
}) => {
  const [registry] = useState(() => CardStyleRegistry.getInstance());
  const [allStyles, setAllStyles] = useState<CardStyle[]>([]);
  const [previewSvgs, setPreviewSvgs] = useState<Record<string, string>>({});

  useEffect(() => {
    const styles = registry.getAllStyles();
    setAllStyles(styles);

    // Generate preview SVGs for all styles with unique IDs
    const svgs: Record<string, string> = {};
    styles.forEach(style => {
      // Generate fresh SVG each time to ensure unique gradient IDs
      svgs[style.id] = registry.generateCardBackSVG(style, 60, 84);
    });
    setPreviewSvgs(svgs);
  }, [registry]);

  // Regenerate previews when selected style changes to ensure proper rendering
  useEffect(() => {
    if (selectedStyleId) {
      const styles = registry.getAllStyles();
      const svgs: Record<string, string> = {};
      styles.forEach(style => {
        svgs[style.id] = registry.generateCardBackSVG(style, 60, 84);
      });
      setPreviewSvgs(svgs);
    }
  }, [selectedStyleId, registry]);

  const categories = [
    { id: 'classic', name: 'Classic', icon: '🎴' },
    { id: 'modern', name: 'Modern', icon: '✨' },
    { id: 'minimal', name: 'Minimal', icon: '⚪' },
    { id: 'fantasy', name: 'Fantasy', icon: '🔮' },
    { id: 'retro', name: 'Retro', icon: '🎰' }
  ] as const;

  const getStylesByCategory = (categoryId: string) => {
    return allStyles.filter(style => style.category === categoryId);
  };

  const selectedStyle = selectedStyleId ? registry.getStyle(selectedStyleId) : null;

  return (
    <div className="space-y-6">
      {/* Current Selection Summary */}
      {selectedStyle && (
        <div className="p-4 bg-gray-700 rounded-lg border border-gray-600">
          <div className="flex items-center gap-4">
            <div 
              className="w-16 h-22 flex-shrink-0 rounded border border-gray-500"
              dangerouslySetInnerHTML={{ __html: previewSvgs[selectedStyle.id] || '' }}
            />
            <div>
              <h4 className="text-white font-medium">{selectedStyle.name}</h4>
              <p className="text-sm text-gray-400 mt-1">{selectedStyle.description}</p>
              <span className="inline-block mt-2 px-2 py-1 text-xs bg-blue-600 text-white rounded">
                {categories.find(c => c.id === selectedStyle.category)?.name || selectedStyle.category}
              </span>
            </div>
          </div>
        </div>
      )}

      {/* Category Sections */}
      {categories.map(category => {
        const categoryStyles = getStylesByCategory(category.id);
        if (categoryStyles.length === 0) return null;

        return (
          <div key={category.id} className="space-y-3">
            <h4 className="text-white font-medium flex items-center gap-2">
              <span>{category.icon}</span>
              <span>{category.name}</span>
            </h4>
            
            <div className="grid grid-cols-3 gap-3">
              {categoryStyles.map(style => {
                const isSelected = selectedStyleId === style.id;
                
                return (
                  <button
                    key={style.id}
                    onClick={() => onStyleSelect(style.id)}
                    className={`
                      group relative p-3 rounded-lg border-2 transition-all duration-200
                      ${isSelected 
                        ? 'border-blue-500 bg-blue-500/10' 
                        : 'border-gray-600 hover:border-gray-500 bg-gray-800/50 hover:bg-gray-700/50'
                      }
                    `}
                  >
                    {/* Card Preview */}
                    <div className="flex justify-center mb-2">
                      <div 
                        className={`
                          w-12 h-16 transition-transform duration-200 group-hover:scale-105
                          ${isSelected ? 'ring-2 ring-blue-400 ring-offset-2 ring-offset-gray-800' : ''}
                        `}
                        style={{ 
                          borderRadius: '6px',
                          overflow: 'hidden',
                          backgroundColor: 'transparent'
                        }}
                        dangerouslySetInnerHTML={{ __html: previewSvgs[style.id] || '' }}
                      />
                    </div>
                    
                    {/* Style Name */}
                    <div className="text-center">
                      <div className={`text-sm font-medium ${isSelected ? 'text-blue-300' : 'text-white'}`}>
                        {style.name}
                      </div>
                      <div className="text-xs text-gray-400 mt-1 line-clamp-2">
                        {style.description}
                      </div>
                    </div>
                    
                    {/* Selection Indicator */}
                    {isSelected && (
                      <div className="absolute top-2 right-2 w-5 h-5 bg-blue-500 rounded-full flex items-center justify-center">
                        <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                          <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                        </svg>
                      </div>
                    )}
                  </button>
                );
              })}
            </div>
          </div>
        );
      })}

      {/* Help Text */}
      <div className="text-xs text-gray-400 mt-4 p-3 bg-gray-800 rounded">
        <p className="mb-1">💡 <strong>Tip:</strong> Card styles affect both the front design and card back patterns.</p>
        <p>Your selected style will be visible to other players when cards are face-down.</p>
      </div>
    </div>
  );
};

export default CardStyleSelector;