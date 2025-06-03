#!/bin/bash

echo "Setting up testing environment for React client..."

# Install test dependencies
echo "Installing test dependencies..."
pnpm add -D vitest@^2.2.0 jsdom@^26.0.0 @testing-library/react@^16.1.0 @testing-library/jest-dom@^6.6.3 @testing-library/user-event@^14.5.2

# Add test scripts to package.json
echo "Adding test scripts to package.json..."
cat package.json | jq '.scripts.test = "vitest"' | jq '.scripts["test:ui"] = "vitest --ui"' | jq '.scripts["test:coverage"] = "vitest --coverage"' > package_tmp.json && mv package_tmp.json package.json

echo "Test environment setup complete!"
echo ""
echo "You can now run:"
echo "  pnpm test              # Run tests"
echo "  pnpm test:ui           # Run tests with UI"
echo "  pnpm test:coverage     # Run tests with coverage"
echo ""
echo "The following test files have been created:"
echo "  - src/ws/__tests__/useLobbyWebSocket.test.ts"
echo "  - src/components/__tests__/GameView.test.tsx" 
echo "  - src/components/__tests__/GameZones.test.tsx"
echo "  - src/__tests__/TicTacToeGameFlow.integration.test.tsx"
echo ""
echo "These tests cover:"
echo "  ✓ WebSocket message handling and patch application"
echo "  ✓ Turn detection and state synchronization"
echo "  ✓ Board rendering and cell click handling"
echo "  ✓ Game flow from start to finish"
echo "  ✓ Error handling and edge cases"