/**
 * Token Registry - Central registry for all available tokens
 * 
 * Manages token definitions, loading, and caching for the customization system.
 */

export interface TokenDefinition {
  id: string;
  name: string;
  category: 'basic' | 'fun' | 'premium' | 'custom';
  svg: string; // SVG content or path
  colorizable: boolean; // Can accept color filters
  defaultColor?: string; // For non-colorizable tokens
  previewScale?: number; // Scale factor for preview display
}

export interface TokenSet {
  id: string;
  name: string;
  description: string;
  tokens: TokenDefinition[];
}

// Basic geometric shapes that work well with color filters
const BASIC_TOKENS: TokenDefinition[] = [
  {
    id: 'circle',
    name: 'Circle',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <circle cx="50" cy="50" r="45" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'square',
    name: 'Square',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <rect x="10" y="10" width="80" height="80" rx="8" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'triangle',
    name: 'Triangle',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50 10 L90 85 L10 85 Z" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'diamond',
    name: 'Diamond',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50 5 L90 50 L50 95 L10 50 Z" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'star',
    name: 'Star',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50 5 L61 39 L95 39 L68 61 L79 95 L50 73 L21 95 L32 61 L5 39 L39 39 Z" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'hexagon',
    name: 'Hexagon',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50 5 L85 25 L85 75 L50 95 L15 75 L15 25 Z" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'cross',
    name: 'Cross',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M35 10 L65 10 L65 35 L90 35 L90 65 L65 65 L65 90 L35 90 L35 65 L10 65 L10 35 L35 35 Z" fill="currentColor" stroke="none"/>
    </svg>`
  },
  {
    id: 'ring',
    name: 'Ring',
    category: 'basic',
    colorizable: true,
    svg: `<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50 5 A45 45 0 1 1 49.9 5 Z M50 25 A25 25 0 1 0 50.1 25 Z" fill="currentColor" fill-rule="evenodd" stroke="none"/>
    </svg>`
  }
];

// Fun tokens - these will be added in Phase 4
const FUN_TOKENS: TokenDefinition[] = [
  // Placeholder for future implementation
];

export class TokenRegistry {
  private static instance: TokenRegistry;
  private tokens: Map<string, TokenDefinition> = new Map();
  private loadedSvgs: Map<string, string> = new Map();
  
  private constructor() {
    this.initializeTokens();
  }
  
  static getInstance(): TokenRegistry {
    if (!TokenRegistry.instance) {
      TokenRegistry.instance = new TokenRegistry();
    }
    return TokenRegistry.instance;
  }
  
  private initializeTokens() {
    // Register all basic tokens
    BASIC_TOKENS.forEach(token => {
      this.tokens.set(token.id, token);
    });
    
    // Register fun tokens when available
    FUN_TOKENS.forEach(token => {
      this.tokens.set(token.id, token);
    });
    
    // Also register legacy tokens for backward compatibility
    this.registerLegacyTokens();
  }
  
  private registerLegacyTokens() {
    // Map existing token files to new system
    const legacyTokens: TokenDefinition[] = [
      {
        id: 'x',
        name: 'X Mark',
        category: 'basic',
        colorizable: true,
        svg: '/tokens/token_p1.svg' // Path to existing file
      },
      {
        id: 'o',
        name: 'O Mark',
        category: 'basic',
        colorizable: true,
        svg: '/tokens/token_p2.svg' // Path to existing file
      }
    ];
    
    legacyTokens.forEach(token => {
      this.tokens.set(token.id, token);
    });
  }
  
  /**
   * Get all available tokens
   */
  getAllTokens(): TokenDefinition[] {
    return Array.from(this.tokens.values());
  }
  
  /**
   * Get tokens by category
   */
  getTokensByCategory(category: TokenDefinition['category']): TokenDefinition[] {
    return this.getAllTokens().filter(token => token.category === category);
  }
  
  /**
   * Get a specific token by ID
   */
  getToken(id: string): TokenDefinition | undefined {
    return this.tokens.get(id);
  }
  
  /**
   * Get token sets for UI display
   */
  getTokenSets(): TokenSet[] {
    return [
      {
        id: 'basic',
        name: 'Basic Shapes',
        description: 'Simple geometric shapes',
        tokens: this.getTokensByCategory('basic')
      },
      {
        id: 'fun',
        name: 'Fun Tokens',
        description: 'Playful and decorative tokens',
        tokens: this.getTokensByCategory('fun')
      }
    ].filter(set => set.tokens.length > 0);
  }
  
  /**
   * Load SVG content for a token
   */
  async loadTokenSvg(tokenId: string): Promise<string> {
    const token = this.tokens.get(tokenId);
    if (!token) {
      throw new Error(`Token ${tokenId} not found`);
    }
    
    // Check cache first
    if (this.loadedSvgs.has(tokenId)) {
      return this.loadedSvgs.get(tokenId)!;
    }
    
    let svgContent: string;
    
    // If SVG is a path, load from file
    if (token.svg.startsWith('/') || token.svg.startsWith('http')) {
      try {
        const response = await fetch(token.svg);
        svgContent = await response.text();
      } catch (error) {
        console.error(`Failed to load token SVG from ${token.svg}:`, error);
        // Fall back to a simple circle
        svgContent = BASIC_TOKENS[0].svg;
      }
    } else {
      // SVG is inline content
      svgContent = token.svg;
    }
    
    // Cache the loaded SVG
    this.loadedSvgs.set(tokenId, svgContent);
    return svgContent;
  }
  
  /**
   * Get default token for new users
   */
  getDefaultToken(): TokenDefinition {
    return this.tokens.get('circle') || BASIC_TOKENS[0];
  }
  
  /**
   * Register a custom token (for future use)
   */
  registerCustomToken(token: TokenDefinition): void {
    if (token.category !== 'custom') {
      throw new Error('Custom tokens must have category "custom"');
    }
    this.tokens.set(token.id, token);
  }
}