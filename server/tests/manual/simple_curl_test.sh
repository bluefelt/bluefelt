#!/bin/bash

echo "Testing basic lobby API with curl..."

echo "1. Testing GET /api/lobbies with 5 second timeout..."
curl --max-time 5 http://localhost:8000/api/lobbies 2>&1

echo ""
echo "2. Test complete."