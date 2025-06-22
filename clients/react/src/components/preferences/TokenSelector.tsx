/**
 * Token Selector Component - Grid display for choosing tokens
 */

import React, { useEffect, useState } from 'react';
import { useAvailableTokens } from '../../hooks/useTokens';
import { TokenManager } from '../../tokens/TokenManager';
import { getColorScheme } from '../../tokens/ColorSchemes';
import { usePlayerPreferences } from '../../context/PlayerPreferencesContext';

interface TokenSelectorProps {
  selectedTokenId: string;
  onTokenSelect: (tokenId: string) => void;
}

interface TokenPreview {
  id: string;
  name: string;
  svg: string;
  category: string;
}

const TokenSelector: React.FC<TokenSelectorProps> = ({ selectedTokenId, onTokenSelect }) => {
  const { tokens, loading } = useAvailableTokens();
  const { preferences } = usePlayerPreferences();
  const [tokenPreviews, setTokenPreviews] = useState<TokenPreview[]>([]);
  const [loadingPreviews, setLoadingPreviews] = useState(true);
  
  useEffect(() => {
    if (tokens.length === 0) return;
    
    const loadPreviews = async () => {
      const manager = TokenManager.getInstance();
      const scheme = getColorScheme(preferences?.colorSchemeId || 'warm');
      const color = scheme?.playerColor || '#FFFFFF';
      
      const previews = await Promise.all(
        tokens.map(async (token) => {
          try {
            const svg = await manager.getColoredSvg(token.id, color);
            return {
              id: token.id,
              name: token.name,
              svg,
              category: token.category
            };
          } catch (error) {
            console.error(`Failed to load preview for token ${token.id}:`, error);
            return null;
          }
        })
      );
      
      setTokenPreviews(previews.filter(Boolean) as TokenPreview[]);
      setLoadingPreviews(false);
    };
    
    loadPreviews();
  }, [tokens, preferences?.colorSchemeId]);
  
  if (loading || loadingPreviews) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white"></div>
      </div>
    );
  }
  
  // Group tokens by category
  const tokensByCategory = tokenPreviews.reduce((acc, token) => {
    if (!acc[token.category]) acc[token.category] = [];
    acc[token.category].push(token);
    return acc;
  }, {} as Record<string, TokenPreview[]>);
  
  const categoryOrder = ['basic', 'fun', 'premium', 'custom'];
  const sortedCategories = Object.keys(tokensByCategory).sort(
    (a, b) => categoryOrder.indexOf(a) - categoryOrder.indexOf(b)
  );
  
  return (
    <div className="space-y-6">
      {sortedCategories.map(category => (
        <div key={category}>
          <h4 className="text-sm font-medium text-gray-400 uppercase tracking-wider mb-3">
            {category} Tokens
          </h4>
          <div className="grid grid-cols-4 gap-3">
            {tokensByCategory[category].map(token => (
              <button
                key={token.id}
                onClick={() => onTokenSelect(token.id)}
                className={`relative p-4 rounded-lg border-2 transition-all ${
                  selectedTokenId === token.id
                    ? 'border-blue-500 bg-blue-500 bg-opacity-20'
                    : 'border-gray-600 hover:border-gray-500 bg-gray-700 hover:bg-gray-600'
                }`}
                aria-label={`Select ${token.name} token`}
              >
                <div 
                  className="w-12 h-12 mx-auto mb-2"
                  dangerouslySetInnerHTML={{ __html: token.svg }}
                />
                <div className="text-xs text-center text-gray-300">
                  {token.name}
                </div>
                {selectedTokenId === token.id && (
                  <div className="absolute top-1 right-1">
                    <svg className="w-4 h-4 text-blue-400" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  </div>
                )}
              </button>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
};

export default TokenSelector;