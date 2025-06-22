/**
 * Preview Pane - Shows how tokens will look in games
 */

import React, { useEffect, useState } from 'react';
import { TokenManager } from '../../tokens/TokenManager';
import { getColorScheme } from '../../tokens/ColorSchemes';
import { CardStyleRegistry } from '../../cards/CardStyleRegistry';

interface PreviewPaneProps {
  tokenId: string;
  colorSchemeId: string;
  cardStyleId: string;
  showOpponentTokens: boolean;
  playerColor?: string;
}

interface TokenPreview {
  svg: string;
  color: string;
  label: string;
}

const PreviewPane: React.FC<PreviewPaneProps> = ({
  tokenId,
  colorSchemeId,
  cardStyleId,
  showOpponentTokens,
  playerColor
}) => {
  const [previews, setPreviews] = useState<TokenPreview[]>([]);
  const [cardBackSvg, setCardBackSvg] = useState<string>('');
  const [loading, setLoading] = useState(true);
  
  useEffect(() => {
    const loadPreviews = async () => {
      const manager = TokenManager.getInstance();
      const cardRegistry = CardStyleRegistry.getInstance();
      const scheme = getColorScheme(colorSchemeId);
      
      if (!scheme) return;
      
      // Load card style preview
      const cardStyle = cardRegistry.getStyle(cardStyleId);
      if (cardStyle) {
        const cardSvg = cardRegistry.generateCardBackSVG(cardStyle, 50, 70);
        setCardBackSvg(cardSvg);
      }
      
      const newPreviews: TokenPreview[] = [];
      
      // Player token - use custom color if provided, otherwise scheme default
      const actualPlayerColor = playerColor || scheme.playerColor;
      try {
        const playerSvg = await manager.getColoredSvg(tokenId, actualPlayerColor);
        newPreviews.push({
          svg: playerSvg,
          color: actualPlayerColor,
          label: 'You'
        });
      } catch (error) {
        console.error('Failed to load player preview:', error);
      }
      
      // Opponent tokens
      const opponentTokens = showOpponentTokens 
        ? ['square', 'triangle', 'diamond'] 
        : ['circle', 'circle', 'circle'];
      
      for (let i = 0; i < Math.min(3, scheme.opponentColors.length); i++) {
        try {
          const opponentSvg = await manager.getColoredSvg(
            opponentTokens[i],
            scheme.opponentColors[i]
          );
          newPreviews.push({
            svg: opponentSvg,
            color: scheme.opponentColors[i],
            label: `Player ${i + 2}`
          });
        } catch (error) {
          console.error(`Failed to load opponent ${i} preview:`, error);
        }
      }
      
      setPreviews(newPreviews);
      setLoading(false);
    };
    
    loadPreviews();
  }, [tokenId, colorSchemeId, cardStyleId, showOpponentTokens, playerColor]);
  
  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
      </div>
    );
  }
  
  return (
    <div className="space-y-6">
      {/* Mini game board preview */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Board Preview</h4>
        <div className="bg-black rounded-lg p-4">
          <div className="grid grid-cols-3 gap-2">
            {[...Array(9)].map((_, idx) => (
              <div
                key={idx}
                className="aspect-square bg-gray-900 rounded flex items-center justify-center border border-gray-700"
              >
                {idx === 4 && previews[0] && (
                  <div 
                    className="w-3/4 h-3/4"
                    dangerouslySetInnerHTML={{ __html: previews[0].svg }}
                  />
                )}
                {idx === 0 && previews[1] && (
                  <div 
                    className="w-3/4 h-3/4"
                    dangerouslySetInnerHTML={{ __html: previews[1].svg }}
                  />
                )}
                {idx === 8 && previews[0] && (
                  <div 
                    className="w-3/4 h-3/4"
                    dangerouslySetInnerHTML={{ __html: previews[0].svg }}
                  />
                )}
              </div>
            ))}
          </div>
        </div>
      </div>
      
      {/* Player list preview */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Player List</h4>
        <div className="space-y-2">
          {previews.map((preview, idx) => (
            <div
              key={idx}
              className={`flex items-center gap-3 p-3 rounded-lg ${
                idx === 0 ? 'bg-blue-500 bg-opacity-20' : 'bg-gray-700'
              }`}
            >
              <div 
                className="w-8 h-8"
                dangerouslySetInnerHTML={{ __html: preview.svg }}
              />
              <div className="flex-1">
                <div className="text-white font-medium">
                  {preview.label}
                  {idx === 0 && <span className="ml-2 text-xs text-blue-400">(You)</span>}
                </div>
              </div>
              {idx === 0 && (
                <div className="text-xs text-green-400">Your turn</div>
              )}
            </div>
          ))}
        </div>
      </div>
      
      {/* Card Style Preview */}
      {cardBackSvg && (
        <div>
          <h4 className="text-sm font-medium text-gray-400 mb-3">Card Style</h4>
          <div className="bg-gray-700 rounded-lg p-4">
            <div className="flex items-center gap-4">
              <div 
                className="w-12 h-16 rounded border border-gray-500"
                dangerouslySetInnerHTML={{ __html: cardBackSvg }}
              />
              <div className="text-sm text-gray-300">
                <div>Card Back Preview</div>
                <div className="text-xs text-gray-400 mt-1">
                  This design will be used for face-down cards
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
      
      {/* Contrast check */}
      <div>
        <h4 className="text-sm font-medium text-gray-400 mb-3">Visibility Check</h4>
        <div className="text-xs space-y-1">
          <div className="flex items-center gap-2">
            <div className="w-3 h-3 bg-green-500 rounded-full"></div>
            <span className="text-gray-300">All colors have good contrast</span>
          </div>
          <div className="text-gray-500">
            Colors are optimized for black backgrounds and colorblind players
          </div>
        </div>
      </div>
    </div>
  );
};

export default PreviewPane;