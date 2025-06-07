const WebSocket = require('ws');

async function testConnect4ActionMap() {
  // Create lobby
  const response = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'connect-four' })
  });
  
  const lobby = await response.json();
  console.log('Created lobby:', lobby.id);
  
  // Track game state
  let actionMapFound = false;
  let gameStarted = false;
  
  // Connect Alice
  const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
  
  ws1.on('message', (data) => {
    const msg = JSON.parse(data);
    
    // Handle welcome message
    if (msg.type === 'welcome') {
      console.log('Alice connected as', msg.you);
    }
    
    // Handle game started
    if (msg.type === 'gameStarted') {
      console.log('\nGame started!');
      gameStarted = true;
      
      // Check if actionMap is in the initial game state
      if (msg.ui?.actionMap) {
        console.log('\nAction map in gameStarted message:');
        logActionMap(msg.ui.actionMap);
      }
    }
    
    // Handle diff messages (patches)
    if (msg.type === 'diff' && !actionMapFound) {
      const patches = msg.patch || [];
      
      // Look for action map patch
      const actionMapPatch = patches.find(p => p.path === '/ui/actionMap');
      if (actionMapPatch) {
        actionMapFound = true;
        console.log('\nAction map from diff message:');
        logActionMap(actionMapPatch.value);
      }
    }
  });
  
  // Connect Bob after a delay
  setTimeout(async () => {
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    ws2.on('open', () => {
      console.log('Bob connected');
      
      // Start game after both players connect
      setTimeout(() => {
        console.log('\nStarting game...');
        ws1.send(JSON.stringify({ action: 'start_game' }));
      }, 500);
    });
    
    // Clean up after 3 seconds
    setTimeout(() => {
      ws1.close();
      ws2.close();
      console.log('\nTest complete');
      process.exit(0);
    }, 3000);
  }, 500);
}

function logActionMap(actionMap) {
  console.log('\n=== FULL ACTION MAP ===');
  console.log(JSON.stringify(actionMap, null, 2));
  
  console.log('\n=== ACTION MAP ANALYSIS ===');
  
  // Group by action type
  const actionTypes = {};
  Object.entries(actionMap).forEach(([key, value]) => {
    const action = value.action || 'unknown';
    if (!actionTypes[action]) {
      actionTypes[action] = [];
    }
    actionTypes[action].push({ key, value });
  });
  
  // Show actions by type
  Object.entries(actionTypes).forEach(([action, entries]) => {
    console.log(`\n${action} actions (${entries.length} total):`);
    entries.slice(0, 3).forEach(({ key, value }) => {
      console.log(`  ${key}: ${JSON.stringify(value)}`);
    });
    if (entries.length > 3) {
      console.log(`  ... and ${entries.length - 3} more`);
    }
  });
  
  // Check for column-based actions
  console.log('\n=== COLUMN ACTION FORMAT ===');
  for (let col = 0; col < 7; col++) {
    // Check different possible key formats
    const possibleKeys = [
      `/zones/board/${col}`,           // Column index only
      `/zones/board/0/${col}`,         // Row 0, column
      `/zones/board/col/${col}`,       // Explicit column
      `/zones/board/column/${col}`     // Full word
    ];
    
    possibleKeys.forEach(key => {
      if (actionMap[key]) {
        console.log(`Found column ${col} at key: ${key}`);
        console.log(`  Action: ${JSON.stringify(actionMap[key])}`);
      }
    });
  }
}

// Run test
testConnect4ActionMap().catch(console.error);