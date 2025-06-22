# Clients Overview

Bluefelt clients are the user-facing applications that connect to the Bluefelt server to enable players to enjoy turn-based multiplayer games. This document provides an overview of Bluefelt's client architecture and philosophy.

## The "Dumb Client" Philosophy

A core principle of Bluefelt's design is that **clients should be "dumb"** - they should contain no game logic whatsoever. This design philosophy has several critical benefits:

### What Makes a Client "Dumb"?

1. **No Game Rules** - Clients don't know or enforce any game rules
2. **No State Validation** - Clients simply display what the server tells them
3. **No Turn Logic** - Clients don't determine whose turn it is
4. **No Win Conditions** - Clients don't calculate who wins

Instead, clients are responsible only for:
- **Rendering** - Displaying the current game state
- **Input Collection** - Capturing player interactions
- **Communication** - Sending actions to and receiving updates from the server

### Benefits of Dumb Clients

This separation provides powerful advantages:

1. **Universal Game Support** - A single client can play ANY Bluefelt game without modification
2. **Guaranteed Consistency** - All players see the exact same game state
3. **Anti-Cheat** - Players cannot manipulate game rules or state
4. **Simplified Development** - Client developers focus on UI/UX, not game logic
5. **Cross-Platform Parity** - Different clients (2D, VR, mobile) show the same game

### The Server's Role

The Bluefelt server is the single source of truth for:
- Game state
- Available actions
- Rule enforcement
- Turn management
- Win conditions

Clients simply render what the server tells them and relay player actions back to the server.

## Communication Protocol

All Bluefelt clients communicate with the server using:
- **WebSocket** for real-time bidirectional communication
- **JSON** for message encoding
- **JSON Patch** for efficient state synchronization

This standardized protocol ensures any client implementation can work with any Bluefelt server.

## Client Documentation

### Core Concepts
- **[Implementing a Client](./clients-implementing.md)** - Technical guide for building a new Bluefelt client

### Official Clients
- **[React Client](./clients-react.md)** - Our reference web implementation
- **[Unity Client](./clients-unity.md)** - Future VR/3D implementation (planned)

## Design Principles

When building a Bluefelt client, follow these principles:

1. **Data-Driven Rendering** - Let server data drive all UI decisions
2. **Generic Components** - Build reusable components that work for any game
3. **Progressive Enhancement** - Start simple, add polish without breaking compatibility
4. **Accessibility First** - Ensure all players can enjoy the games
5. **Performance Matters** - Smooth, responsive interactions enhance gameplay

## Future Vision

The separation of game logic from presentation enables exciting possibilities:

- **Multiple Visual Styles** - Same game, different artistic presentations
- **Adaptive Interfaces** - UI that adjusts to player preferences and abilities
- **Cross-Reality Play** - VR players competing with mobile players
- **AI Assistance** - Clients that help players understand complex game states
- **Spectator Modes** - Specialized clients for watching games

By keeping clients "dumb" and focused purely on presentation, Bluefelt ensures that innovation in client design never compromises game integrity or fairness.