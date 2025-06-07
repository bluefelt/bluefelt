# Automated End-to-End Testing for Bluefelt

This document describes the automated testing workflow developed for rapid iteration and debugging of Bluefelt games.

## Overview

Instead of manual testing through the browser, we can programmatically:
- Start/stop the server
- Create lobbies via HTTP API
- Connect players via WebSocket
- Start games and inspect state
- Simulate player actions
- Capture detailed logs and game state

## Benefits

✅ **Speed**: Test cycles take seconds instead of minutes  
✅ **Reproducibility**: Exact same test conditions every time  
✅ **Detailed Inspection**: Full access to game state and server logs  
✅ **Automation**: Can test multiple scenarios without manual intervention  
✅ **Debugging**: Immediate access to both client and server perspectives  

## Core Testing Script

### Basic Setup (`test_go_fish.py`)

```python
#!/usr/bin/env python3
import asyncio
import websockets
import json
import requests

async def test_go_fish():
    # 1. Create lobby via HTTP API
    response = requests.post("http://localhost:8000/api/lobbies", 
                           json={"game_id": "go-fish", "max_players": 4})
    lobby_data = response.json()
    lobby_id = lobby_data["id"]
    print(f"Created lobby: {lobby_id}")
    
    # 2. Connect players via WebSocket
    alice_uri = f"ws://localhost:8000/api/lobbies/{lobby_id}/ws?player_id=alice&join=true"
    bob_uri = f"ws://localhost:8000/api/lobbies/{lobby_id}/ws?player_id=bob&join=true"
    
    async def connect_player(uri, name):
        async with websockets.connect(uri) as websocket:
            print(f"{name} connected")
            
            # 3. Read and analyze game state messages
            for i in range(10):
                try:
                    message = await asyncio.wait_for(websocket.recv(), timeout=3.0)
                    data = json.loads(message)
                    
                    # 4. Extract and display relevant game state
                    if 'zones' in data:
                        zones = data['zones']
                        pool_cards = len(zones.get('pool', {}).get('items', []))
                        p1_cards = len(zones.get('hand_p1', {}).get('items', []))
                        p2_cards = len(zones.get('hand_p2', {}).get('items', []))
                        print(f"  CARDS: Pool: {pool_cards}, P1: {p1_cards}, P2: {p2_cards}")
                    
                    # 5. Trigger game actions programmatically
                    if name == "bob" and "players" in data and len(data.get("players", [])) >= 2:
                        await websocket.send(json.dumps({"type": "startGame"}))
                        
                except asyncio.TimeoutError:
                    break
    
    # 6. Run both players concurrently
    await asyncio.gather(
        connect_player(alice_uri, "Alice"),
        connect_player(bob_uri, "Bob")
    )

if __name__ == "__main__":
    asyncio.run(test_go_fish())
```

## Usage Workflow

### 1. Start Server in Background
```bash
cd /home/fancymatt/Code/bluefelt/server
cargo run > server.log 2>&1 &
```

### 2. Run Test Script
```bash
# Setup virtual environment (one-time)
python3 -m venv venv
source venv/bin/activate
pip install websockets requests

# Run tests
python3 test_go_fish.py
```

### 3. Analyze Results
```bash
# Check server logs for debug info
tail -f server.log
grep "DEBUG\|ERROR" server.log

# Check specific events
grep -A 5 -B 5 "enterActions\|dealCards" server.log
```

## Key API Endpoints

### HTTP API
- `POST /api/lobbies` - Create lobby
- `GET /api/lobbies` - List lobbies  
- `GET /api/lobbies/{id}` - Get lobby state
- `GET /api/games` - List available games

### WebSocket API
- `GET /api/lobbies/{id}/ws?player_id={name}&join=true` - Join lobby
- Messages: `{"type": "startGame"}`, `{"type": "action", "action": "...", ...}`

## Real Example: Go Fish Card Dealing Debug

### Problem Discovery
Our test revealed that Go Fish wasn't dealing cards to players:

```
CARD COUNT: Pool: 52, P1: 0, P2: 0, P3: 0, P4: 0
```

### Root Cause Analysis
The automated test showed:
1. ✅ Server loads 52 + 4 entities correctly
2. ✅ Pool contains all 52 cards
3. ❌ Player hands are empty
4. ❌ No `process_phases` debug messages in logs

**Conclusion**: `enterActions` aren't being executed when the game starts.

### Server Log Analysis
```bash
grep -A 10 -B 5 "process_phases\|enterActions" server.log
```
Revealed: No phase processing happening during game startup.

## Advanced Testing Patterns

### 1. State Inspection Helper
```python
def analyze_game_state(data, player_name):
    """Extract and display key game state information"""
    if 'zones' not in data:
        return
        
    zones = data['zones']
    pool_size = len(zones.get('pool', {}).get('items', []))
    
    # Check all player hands
    hands = {}
    for i in range(1, 5):
        hand_key = f'hand_p{i}'
        if hand_key in zones:
            hands[f'P{i}'] = len(zones[hand_key].get('items', []))
    
    print(f"{player_name} sees: Pool={pool_size}, Hands={hands}")
    
    # Check action map
    if 'actionMap' in data:
        actions = data['actionMap']
        for player, action_map in actions.items():
            if action_map:
                print(f"  {player} actions: {list(action_map.keys())}")
```

### 2. Multi-Player Test
```python
async def test_multiplayer_scenario():
    """Test with 3-4 players to verify scaling"""
    lobby_id = create_lobby("go-fish")
    
    players = ["alice", "bob", "charlie", "diana"]
    tasks = []
    
    for player in players:
        uri = f"ws://localhost:8000/api/lobbies/{lobby_id}/ws?player_id={player}&join=true"
        tasks.append(connect_player(uri, player))
    
    await asyncio.gather(*tasks)
```

### 3. Action Testing
```python
async def test_player_actions():
    """Test specific game actions"""
    # Connect and start game...
    
    # Simulate player action
    action_message = {
        "type": "action",
        "action": "selectRank", 
        "args": {"rank": "A"}
    }
    await websocket.send(json.dumps(action_message))
    
    # Wait for response and validate
    response = await websocket.recv()
    # Analyze response...
```

### 4. Performance Testing
```python
async def test_rapid_lobbies():
    """Test creating many lobbies quickly"""
    for i in range(10):
        lobby_id = create_lobby("tic-tac-toe")
        print(f"Created lobby {i}: {lobby_id}")
```

## Testing Different Games

### Quick Game Validation
```python
games = ["tic-tac-toe", "connect-four", "three-mens-morris", "go-fish"]
for game in games:
    print(f"\n=== Testing {game} ===")
    await test_game_basic_flow(game)
```

### Game-Specific Tests
```python
async def test_tic_tac_toe():
    """Test tic-tac-toe specific functionality"""
    # Create 2-player lobby
    # Place marks on board
    # Verify win detection
    
async def test_connect_four():
    """Test Connect-4 gravity mechanics"""
    # Test gravity placement
    # Test win detection
```

## Debugging Techniques

### 1. Message Capture
```python
# Save all messages for analysis
messages = []
async for message in websocket:
    data = json.loads(message)
    messages.append(data)
    
# Analyze patterns
for i, msg in enumerate(messages):
    print(f"Message {i}: {msg.get('type', 'unknown')}")
```

### 2. Server Log Correlation
```python
import subprocess
import time

def get_recent_logs():
    """Get recent server log entries"""
    result = subprocess.run(['tail', '-20', 'server.log'], 
                          capture_output=True, text=True)
    return result.stdout
```

### 3. State Diff Analysis
```python
def compare_states(before, after):
    """Compare game states to see what changed"""
    if 'zones' in before and 'zones' in after:
        for zone_id in before['zones']:
            before_items = before['zones'][zone_id].get('items', [])
            after_items = after['zones'][zone_id].get('items', [])
            if before_items != after_items:
                print(f"Zone {zone_id}: {len(before_items)} -> {len(after_items)}")
```

## Best Practices

### 1. Test Isolation
- Create new lobby for each test
- Use unique player names
- Clean up resources

### 2. Error Handling
```python
try:
    await websocket.send(message)
    response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
except websockets.exceptions.ConnectionClosed:
    print("Connection lost")
except asyncio.TimeoutError:
    print("No response received")
```

### 3. Concurrent Testing
```python
# Test multiple scenarios simultaneously
await asyncio.gather(
    test_go_fish(),
    test_tic_tac_toe(),
    test_connect_four()
)
```

## Integration with Development

### Pre-commit Testing
```bash
#!/bin/bash
# Run automated tests before committing
python3 test_suite.py
if [ $? -eq 0 ]; then
    echo "✅ All tests passed"
    git commit "$@"
else
    echo "❌ Tests failed"
    exit 1
fi
```

### Continuous Debugging
```bash
# Watch for changes and re-run tests
while inotifywait -e modify src/; do
    cargo build && python3 test_go_fish.py
done
```

## Future Enhancements

1. **Test Framework**: Convert to pytest for better structure
2. **Assertions**: Add proper test assertions and reporting
3. **Coverage**: Track which game features are tested
4. **Performance**: Measure response times and throughput
5. **Regression**: Save test scenarios for regression testing

## Conclusion

This automated testing approach transforms the development experience:

- **Before**: Manual browser testing, slow feedback loops
- **After**: Instant programmatic testing with detailed insights

The combination of HTTP API + WebSocket + server logs provides complete visibility into the system behavior, enabling rapid iteration and confident debugging.