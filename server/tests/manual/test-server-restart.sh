#!/bin/bash

echo "Testing server restart scenario..."
echo "1. Open browser to http://localhost:5173"
echo "2. Login and create a new lobby"
echo "3. Note the lobby URL (e.g., http://localhost:5173/lobby/xxx)"
echo "4. Press Enter to kill the server..."
read

# Kill the server
echo "Killing server..."
pkill -f "target/debug/bluefelt-core"

echo "Server killed. The browser should now show an error when trying to interact with the game."
echo "Check that:"
echo "  - The error is handled gracefully"
echo "  - User is redirected with an informative message"
echo "  - The message appears on the lobbies or login page"
echo ""
echo "Press Enter to restart the server..."
read

# Restart the server
echo "Restarting server..."
cd /home/fancymatt/Code/bluefelt/server && cargo run &

echo "Server restarted. You can now create a new lobby."