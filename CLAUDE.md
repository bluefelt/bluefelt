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
- `manifest.yaml` - Game metadata - **IMPORTANT: Required fields are gameId, version, specVersion, and metadata (with name, description, author, players)**
- `entities.yaml` - Game rules including:
  - Entities (pieces, cards, tokens)
  - Zones (areas where entities exist)
  - Actions (player actions)
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
- Server provides an action map that directly maps locations to available actions
- Action map structure: `{"/zones/board/0/0": {"action": "place", "direction": "Choose a cell"}, ...}`
- Location paths follow a consistent format:
  - Grid zones: `/zones/{zoneId}/{row}/{col}`
  - List/deck zones: `/zones/{zoneId}/{index}`
  - Whole zones: `/zones/{zoneId}`
- Client renders zones as interactive components where players can click to execute actions
- UI directions are defined in the game's `actions.yaml` under `ui.direction`
- Supports multiple zones and actions per game without hardcoding assumptions

### Custom Hooks and Game End Detection
- Actions can trigger other actions using the `triggers` field in `actions.yaml`
- Custom hooks can be defined with `hook: hookName` and `auto: true` for automatic actions
- Game end detection for tic-tac-toe checks for wins (3 in a row) and ties (board full)
- Game status stored in `/meta/gameStatus` with fields: `state`, `winner`, `tie`
- When game ends, turn advancement is prevented and UI shows game result
- WebAssembly hooks can be compiled and placed as `hooks.wasm` in game directory