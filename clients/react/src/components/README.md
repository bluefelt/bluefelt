# React Components Documentation

## Overview

This directory contains all React components for the Bluefelt game platform. The components are organized by feature and follow Clean Code principles.

## Directory Structure

```
components/
├── ui/               # Reusable UI components
│   ├── Button.tsx    # Consistent button component
│   └── Card.tsx      # Card container component
├── zones/            # Game zone components
│   ├── Board.tsx     # Main board component (orchestrator)
│   ├── BoardZone.tsx # Individual zone renderer
│   └── BoardCell.tsx # Individual cell component
├── GameView.tsx      # Main game view component
├── GameZones.tsx     # Zone orchestrator component
├── GameControls.tsx  # Game action controls
└── ...              # Other game components
```

## Component Guidelines

### 1. Component Size
- Keep components under 300 lines
- Extract logic into custom hooks
- Split large components into smaller, focused ones

### 2. Styling
- Use Tailwind CSS classes
- Avoid inline styles except for dynamic values
- Use theme constants from `src/theme/constants.ts`

### 3. Performance
- Use React.memo for expensive components
- Implement lazy loading for route components
- Avoid console.log statements (use logger utility)

### 4. TypeScript
- Define proper interfaces for all props
- Avoid using `any` type
- Use type imports from `src/types/`

## Key Components

### GameView
The main game orchestrator that manages:
- WebSocket connections
- Game state
- Player actions
- UI coordination

### GameZones
Renders different zone types (board, cards, etc.) based on game configuration.

### Board Components
- **Board**: Main entry point for grid-based zones
- **BoardZone**: Handles individual zone rendering and sizing
- **BoardCell**: Renders individual cells with click handling

## Custom Hooks

### useGameActions
Handles all game action logic:
- Board cell clicks
- Card actions
- Zone interactions

### useMarkColor
Determines entity colors based on player assignments.

## Testing

All components have corresponding test files in `__tests__` directories. Run tests with:

```bash
pnpm test
```

## Adding New Components

1. Create component file in appropriate directory
2. Add TypeScript interfaces
3. Use theme constants for styling
4. Export from barrel file (index.ts)
5. Add tests
6. Update this documentation