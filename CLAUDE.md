# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bluefelt is a platform for creating and playing turn-based multiplayer games. It consists of:
- **Rust server** (`server/`) - WebSocket-based game server with WebAssembly rule enforcement
- **React web client** (`clients/react/`) - TypeScript/React frontend with real-time updates
- **Game definitions** (`games/`) - YAML-based declarative game rules
- **Rust SDK** (`sdk/rust/`) - For writing WebAssembly game hooks
- **Documentation** (`docs/`) - Docusaurus documentation site

## Common Commands

### Server Development
```bash
cd server
cargo build          # Build server
cargo test           # Run tests
cargo run            # Start server (port 3213)
```

### Client Development
```bash
cd clients/react
pnpm install         # Install dependencies
pnpm dev             # Start dev server (port 5173)
pnpm build           # Production build
pnpm lint            # Run linter
pnpm preview         # Preview production build
```

### Documentation
```bash
cd docs
yarn start           # Start docs dev server
yarn build           # Build documentation
yarn typecheck       # TypeScript check
```

### Monorepo Commands (from root)
```bash
yarn nx affected -t lint test build    # Run affected targets
yarn nx run-many -t lint test build    # Run all targets
```

## Architecture Overview

### Game Engine
Games are defined using YAML files in `games/<game-name>/<version>/`:
- `manifest.yaml` - Game metadata (id, name, players)
- `entities.yaml` - Game rules including:
  - Entities (pieces, cards, tokens)
  - Zones (areas where entities exist)
  - Verbs (player actions)
  - Phases (game flow)
  - Setup (initial state)
  - Hooks (WebAssembly functions)

### State Synchronization
- Server maintains authoritative game state
- Client-server communication via WebSocket
- State updates sent as JSON Patches
- Client applies patches to maintain synchronized state

### Key Technologies
- **Server**: Rust, Axum, Tokio, Wasmtime
- **Client**: React, TypeScript, Vite, TanStack Query, Zustand, Tailwind CSS
- **Real-time**: WebSocket with JSON Patch
- **Game Logic**: WebAssembly modules for complex rules

### Interactive Zones
- Server groups possible verbs by type and includes UI directions from game definitions
- Possible verbs structure: `{"verb": "place", "direction": "Choose a cell to place a mark", "validOptions": [{"zone": "board", "row": 0, "col": 0}, ...]}`
- Client renders zones as interactive components where players can click to execute verbs
- UI directions are defined in the game's `verbs.yaml` under `ui.direction`
- Supports multiple zones and verbs per game without hardcoding assumptions