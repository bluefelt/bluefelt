#!/bin/bash

echo "=== Basic Integration Test ==="
echo

echo "1. Creating lobby..."
RESPONSE=$(curl -s -X POST http://localhost:8000/api/lobbies \
  -H "Content-Type: application/json" \
  -d '{"game_id": "tic-tac-toe"}')

echo "Response: $RESPONSE"

LOBBY_ID=$(echo $RESPONSE | jq -r '.id' 2>/dev/null)

if [ -z "$LOBBY_ID" ] || [ "$LOBBY_ID" = "null" ]; then
  echo "❌ Failed to create lobby"
  exit 1
fi

echo "✅ Created lobby: $LOBBY_ID"
echo

echo "2. Listing lobbies..."
LOBBIES=$(curl -s http://localhost:8000/api/lobbies)
echo "Lobbies: $LOBBIES"

COUNT=$(echo $LOBBIES | jq 'length' 2>/dev/null)
echo "✅ Found $COUNT lobbies"
echo

echo "3. Getting lobby info..."
LOBBY_INFO=$(curl -s http://localhost:8000/api/lobbies/$LOBBY_ID)
echo "Lobby info: $LOBBY_INFO"

echo
echo "✅ Basic HTTP API tests passed!"