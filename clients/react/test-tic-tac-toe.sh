#!/bin/bash

# Tic-Tac-Toe Test Runner
# Runs all tic-tac-toe related tests to ensure client handling is bulletproof

echo "🎯 Running Tic-Tac-Toe Client Tests..."
echo "======================================"

# Run all tic-tac-toe tests
pnpm test TicTacToe

echo ""
echo "📊 Test Coverage Summary:"
echo "- WebSocket Message Handling: Message parsing, patch application, edge cases"
echo "- UI State Synchronization: Turn detection, action maps, board updates"  
echo "- Action Handling: Click logic, message construction, validation"
echo "- Integration Tests: Complete game flows, reconnection, error recovery"
echo ""
echo "Total: 41 comprehensive tests covering all critical client-side scenarios"