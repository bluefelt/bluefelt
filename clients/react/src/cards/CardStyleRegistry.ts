/**
 * Card Style Registry - Central system for card customization
 * 
 * Defines all available card styles with both front and back designs.
 * Card styles are data-driven and use programmatic SVG/CSS generation.
 */

export interface CardStyle {
  id: string;
  name: string;
  description: string;
  category: 'classic' | 'modern' | 'fantasy' | 'retro' | 'minimal';
  front: {
    cornerStyle: 'classic' | 'rounded' | 'square' | 'ornate';
    centerStyle: 'large' | 'medium' | 'small' | 'minimal';
    borderStyle: 'none' | 'simple' | 'decorative' | 'elegant';
    backgroundPattern: 'none' | 'subtle' | 'textured' | 'gradient';
    colors: {
      background: string;
      border: string;
      text: string;
      accent?: string;
    };
  };
  back: {
    pattern: 'geometric' | 'floral' | 'abstract' | 'minimal' | 'classic';
    colors: {
      primary: string;
      secondary: string;
      accent?: string;
    };
    complexity: 'low' | 'medium' | 'high';
  };
}

export class CardStyleRegistry {
  private static instance: CardStyleRegistry;
  private styles: Map<string, CardStyle> = new Map();

  private constructor() {
    this.initializeDefaultStyles();
  }

  public static getInstance(): CardStyleRegistry {
    if (!CardStyleRegistry.instance) {
      CardStyleRegistry.instance = new CardStyleRegistry();
    }
    return CardStyleRegistry.instance;
  }

  private initializeDefaultStyles(): void {
    // Classic Style - Traditional playing card look
    this.styles.set('classic', {
      id: 'classic',
      name: 'Classic',
      description: 'Traditional playing card design',
      category: 'classic',
      front: {
        cornerStyle: 'classic',
        centerStyle: 'large',
        borderStyle: 'simple',
        backgroundPattern: 'none',
        colors: {
          background: '#FFFFFF',
          border: '#000000',
          text: '#000000'
        }
      },
      back: {
        pattern: 'classic',
        colors: {
          primary: '#2B5CE6',
          secondary: '#1E40AF'
        },
        complexity: 'medium'
      }
    });

    // Modern Minimal
    this.styles.set('minimal', {
      id: 'minimal',
      name: 'Minimal',
      description: 'Clean, modern design with minimal decoration',
      category: 'minimal',
      front: {
        cornerStyle: 'rounded',
        centerStyle: 'medium',
        borderStyle: 'none',
        backgroundPattern: 'none',
        colors: {
          background: '#FAFAFA',
          border: '#E5E5E5',
          text: '#1A1A1A'
        }
      },
      back: {
        pattern: 'minimal',
        colors: {
          primary: '#6366F1',
          secondary: '#4F46E5'
        },
        complexity: 'low'
      }
    });

    // Dark Theme
    this.styles.set('dark', {
      id: 'dark',
      name: 'Dark',
      description: 'Sleek dark theme for low-light gaming',
      category: 'modern',
      front: {
        cornerStyle: 'rounded',
        centerStyle: 'large',
        borderStyle: 'simple',
        backgroundPattern: 'gradient',
        colors: {
          background: '#1F2937',
          border: '#4B5563',
          text: '#F9FAFB',
          accent: '#3B82F6'
        }
      },
      back: {
        pattern: 'geometric',
        colors: {
          primary: '#111827',
          secondary: '#1F2937',
          accent: '#3B82F6'
        },
        complexity: 'medium'
      }
    });

    // Elegant Gold
    this.styles.set('elegant', {
      id: 'elegant',
      name: 'Elegant Gold',
      description: 'Luxurious design with gold accents',
      category: 'classic',
      front: {
        cornerStyle: 'ornate',
        centerStyle: 'large',
        borderStyle: 'elegant',
        backgroundPattern: 'subtle',
        colors: {
          background: '#FFFEF7',
          border: '#D4AF37',
          text: '#1A1A1A',
          accent: '#B8860B'
        }
      },
      back: {
        pattern: 'floral',
        colors: {
          primary: '#D4AF37',
          secondary: '#B8860B',
          accent: '#FFD700'
        },
        complexity: 'high'
      }
    });

    // Retro Green
    this.styles.set('retro', {
      id: 'retro',
      name: 'Casino Green',
      description: 'Classic casino table green theme',
      category: 'retro',
      front: {
        cornerStyle: 'classic',
        centerStyle: 'large',
        borderStyle: 'decorative',
        backgroundPattern: 'textured',
        colors: {
          background: '#F8F8F8',
          border: '#006B3C',
          text: '#1A1A1A',
          accent: '#228B22'
        }
      },
      back: {
        pattern: 'classic',
        colors: {
          primary: '#006B3C',
          secondary: '#228B22',
          accent: '#32CD32'
        },
        complexity: 'medium'
      }
    });

    // Fantasy Purple
    this.styles.set('fantasy', {
      id: 'fantasy',
      name: 'Mystic Purple',
      description: 'Magical theme with mystical elements',
      category: 'fantasy',
      front: {
        cornerStyle: 'ornate',
        centerStyle: 'medium',
        borderStyle: 'decorative',
        backgroundPattern: 'gradient',
        colors: {
          background: '#F5F3FF',
          border: '#7C3AED',
          text: '#1F2937',
          accent: '#A855F7'
        }
      },
      back: {
        pattern: 'abstract',
        colors: {
          primary: '#7C3AED',
          secondary: '#A855F7',
          accent: '#C084FC'
        },
        complexity: 'high'
      }
    });
  }

  public getStyle(id: string): CardStyle | null {
    return this.styles.get(id) || null;
  }

  public getAllStyles(): CardStyle[] {
    return Array.from(this.styles.values());
  }

  public getStylesByCategory(category: CardStyle['category']): CardStyle[] {
    return this.getAllStyles().filter(style => style.category === category);
  }

  public getDefaultStyle(): CardStyle {
    return this.getStyle('classic')!;
  }

  public addStyle(style: CardStyle): void {
    this.styles.set(style.id, style);
  }

  /**
   * Generate SVG for card back based on style
   */
  public generateCardBackSVG(style: CardStyle, width: number = 70, height: number = 105): string {
    const { back } = style;
    const { pattern, colors, complexity } = back;

    // Generate unique gradient ID to avoid conflicts when multiple SVGs are on the same page
    const gradientId = `cardGradient-${style.id}-${Math.random().toString(36).substr(2, 9)}`;

    let patternContent = '';
    
    switch (pattern) {
      case 'classic':
        patternContent = this.generateClassicPattern(colors, complexity);
        break;
      case 'geometric':
        patternContent = this.generateGeometricPattern(colors, complexity);
        break;
      case 'minimal':
        patternContent = this.generateMinimalPattern(colors, complexity);
        break;
      case 'floral':
        patternContent = this.generateFloralPattern(colors, complexity);
        break;
      case 'abstract':
        patternContent = this.generateAbstractPattern(colors, complexity);
        break;
      default:
        patternContent = this.generateClassicPattern(colors, complexity);
    }

    return `
      <svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">
        <defs>
          <linearGradient id="${gradientId}" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" style="stop-color:${colors.primary};stop-opacity:1" />
            <stop offset="100%" style="stop-color:${colors.secondary};stop-opacity:1" />
          </linearGradient>
        </defs>
        <rect width="100%" height="100%" rx="8" ry="8" fill="url(#${gradientId})" stroke="${colors.accent || colors.secondary}" stroke-width="2"/>
        ${patternContent}
      </svg>
    `;
  }

  private generateClassicPattern(colors: any, complexity: string): string {
    const centerX = 35;
    const centerY = 52.5;
    
    if (complexity === 'low') {
      return `<circle cx="${centerX}" cy="${centerY}" r="15" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="2"/>`;
    }
    
    return `
      <g transform="translate(${centerX}, ${centerY})">
        <circle r="20" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="2"/>
        <circle r="12" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>
        <polygon points="-8,-8 8,-8 8,8 -8,8" fill="${colors.accent || colors.secondary}" opacity="0.3"/>
      </g>
    `;
  }

  private generateGeometricPattern(colors: any, complexity: string): string {
    if (complexity === 'low') {
      return `<rect x="20" y="35" width="30" height="35" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>`;
    }
    
    return `
      <g opacity="0.6">
        <polygon points="35,15 50,35 35,55 20,35" fill="${colors.accent || colors.secondary}" opacity="0.3"/>
        <polygon points="35,25 45,37.5 35,50 25,37.5" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>
        <circle cx="35" cy="37.5" r="8" fill="none" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>
      </g>
    `;
  }

  private generateMinimalPattern(colors: any, complexity: string): string {
    return `
      <g opacity="0.4">
        <line x1="35" y1="20" x2="35" y2="85" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>
        <line x1="15" y1="52.5" x2="55" y2="52.5" stroke="${colors.accent || colors.secondary}" stroke-width="1"/>
      </g>
    `;
  }

  private generateFloralPattern(colors: any, complexity: string): string {
    const centerX = 35;
    const centerY = 52.5;
    
    return `
      <g transform="translate(${centerX}, ${centerY})" opacity="0.5">
        <path d="M-10,-10 Q0,-20 10,-10 Q20,0 10,10 Q0,20 -10,10 Q-20,0 -10,-10 Z" 
              fill="${colors.accent || colors.secondary}" opacity="0.3"/>
        <circle r="5" fill="${colors.accent || colors.secondary}"/>
        <path d="M-5,-15 Q0,-5 5,-15 M-15,-5 Q-5,0 -15,5 M-5,15 Q0,5 5,15 M15,-5 Q5,0 15,5" 
              stroke="${colors.accent || colors.secondary}" stroke-width="1" fill="none"/>
      </g>
    `;
  }

  private generateAbstractPattern(colors: any, complexity: string): string {
    return `
      <g opacity="0.6">
        <path d="M10,20 Q35,10 60,30 Q50,55 25,45 Q15,70 35,85 Q55,75 45,50 Q70,40 50,15" 
              stroke="${colors.accent || colors.secondary}" stroke-width="2" fill="none"/>
        <circle cx="25" cy="35" r="3" fill="${colors.accent || colors.secondary}"/>
        <circle cx="45" cy="60" r="3" fill="${colors.accent || colors.secondary}"/>
        <circle cx="50" cy="25" r="2" fill="${colors.accent || colors.secondary}"/>
      </g>
    `;
  }
}