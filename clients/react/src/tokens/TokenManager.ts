/**
 * Token Manager - Handles token loading, caching, and rendering
 */

import { TokenRegistry, type TokenDefinition } from './TokenRegistry';
import type { ColorOption } from './ColorSchemes';

export interface TokenRenderOptions {
  color?: string;
  size?: number;
  className?: string;
}

export class TokenManager {
  private static instance: TokenManager;
  private registry: TokenRegistry;
  private svgCache: Map<string, string> = new Map();
  private coloredSvgCache: Map<string, string> = new Map();
  
  private constructor() {
    this.registry = TokenRegistry.getInstance();
  }
  
  static getInstance(): TokenManager {
    if (!TokenManager.instance) {
      TokenManager.instance = new TokenManager();
    }
    return TokenManager.instance;
  }
  
  /**
   * Load and cache token SVG
   */
  async loadToken(tokenId: string): Promise<string> {
    // Check cache first
    if (this.svgCache.has(tokenId)) {
      return this.svgCache.get(tokenId)!;
    }
    
    try {
      const svg = await this.registry.loadTokenSvg(tokenId);
      this.svgCache.set(tokenId, svg);
      return svg;
    } catch (error) {
      console.error(`Failed to load token ${tokenId}:`, error);
      // Return a fallback circle
      const fallback = `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
        <circle cx="50" cy="50" r="45" fill="currentColor"/>
      </svg>`;
      this.svgCache.set(tokenId, fallback);
      return fallback;
    }
  }
  
  /**
   * Get SVG with color applied
   */
  async getColoredSvg(tokenId: string, color: string): Promise<string> {
    const cacheKey = `${tokenId}-${color}`;
    
    // Check colored cache
    if (this.coloredSvgCache.has(cacheKey)) {
      return this.coloredSvgCache.get(cacheKey)!;
    }
    
    // Load base SVG
    const baseSvg = await this.loadToken(tokenId);
    const token = this.registry.getToken(tokenId);
    
    if (!token?.colorizable) {
      // Token doesn't support coloring, return as-is
      this.coloredSvgCache.set(cacheKey, baseSvg);
      return baseSvg;
    }
    
    // Apply color by replacing currentColor
    const coloredSvg = baseSvg.replace(/currentColor/g, color);
    
    this.coloredSvgCache.set(cacheKey, coloredSvg);
    return coloredSvg;
  }
  
  /**
   * Create a data URL for a token with color
   */
  async getTokenDataUrl(tokenId: string, color: string): Promise<string> {
    const svg = await this.getColoredSvg(tokenId, color);
    const encoded = encodeURIComponent(svg);
    return `data:image/svg+xml;charset=utf-8,${encoded}`;
  }
  
  /**
   * Render token as React element
   */
  async renderToken(
    tokenId: string, 
    options: TokenRenderOptions = {}
  ): Promise<React.ReactElement> {
    const { color = '#FFFFFF', size = 40, className = '' } = options;
    const dataUrl = await this.getTokenDataUrl(tokenId, color);
    
    return {
      type: 'div',
      props: {
        className: `token-display ${className}`,
        style: {
          width: `${size}px`,
          height: `${size}px`,
          backgroundImage: `url("${dataUrl}")`,
          backgroundSize: 'contain',
          backgroundRepeat: 'no-repeat',
          backgroundPosition: 'center'
        }
      }
    } as any;
  }
  
  /**
   * Get token definition
   */
  getToken(tokenId: string): TokenDefinition | undefined {
    return this.registry.getToken(tokenId);
  }
  
  /**
   * Get all available tokens
   */
  getAllTokens(): TokenDefinition[] {
    return this.registry.getAllTokens();
  }
  
  /**
   * Clear caches (useful when tokens are updated)
   */
  clearCache(): void {
    this.svgCache.clear();
    this.coloredSvgCache.clear();
  }
  
  /**
   * Preload tokens for better performance
   */
  async preloadTokens(tokenIds: string[]): Promise<void> {
    await Promise.all(
      tokenIds.map(id => this.loadToken(id))
    );
  }
  
  /**
   * Convert legacy token types to new system
   */
  mapLegacyToken(legacyType: string): string {
    const mapping: Record<string, string> = {
      'p1': 'x',
      'p2': 'o',
      'x': 'x',
      'o': 'o',
      'circle': 'circle',
      'cross': 'cross'
    };
    
    return mapping[legacyType] || 'circle';
  }
}