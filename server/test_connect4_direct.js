const WebSocket = require('ws');

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testConnect4() {
  // Create lobby
  const response = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'connect-four' })
  });
  
  const lobby = await response.json();
  console.log('Created lobby:', lobby.id);
  
  // Connect Alice
  const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
  await new Promise(resolve => ws1.on('open', resolve));
  console.log('Alice connected');
  
  // Connect Bob
  const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
  await new Promise(resolve => ws2.on('open', resolve));
  console.log('Bob connected');
  
  // Set up message handler for Alice
  let actionMapFound = false;
  ws1.on('message', (data) => {
    const msg = JSON.parse(data);
    
    if (msg.type === 'patch') {
      const patches = msg.patch || [];
      
      // Look for action map patch
      const actionMapPatch = patches.find(p => p.path === '/ui/actionMap');
      if (actionMapPatch && !actionMapFound) {
        actionMapFound = true;
        console.log('\n=== ACTION MAP FOUND ===');
        const actionMap = actionMapPatch.value;
        
        // Show all keys
        console.log('\nAll action map keys:');
        Object.keys(actionMap).forEach(key => {
          console.log(`  ${key}`);
        });
        
        // Show details for column actions
        console.log('\nColumn actions:');
        Object.entries(actionMap).forEach(([key, value]) => {
          if (key.includes('/zones/board/')) {
            console.log(`  ${key}: ${JSON.stringify(value)}`);
          }
        });
        
        // Check first column specifically
        console.log('\nChecking specific columns:');
        for (let col = 0; col < 7; col++) {
          // Try different possible formats
          const keys = [
            `/zones/board/${col}`,
            `/zones/board/0/${col}`,
            `/zones/board/column/${col}`
          ];
          
          keys.forEach(key => {
            if (actionMap[key]) {
              console.log(`  Found action at ${key}: ${JSON.stringify(actionMap[key])}`);
            }
          });
        }
      }
    }
  });
  
  // Wait a bit for initial messages
  await sleep(1000);
  
  // Start the game
  console.log('\nStarting game...');
  ws1.send(JSON.stringify({ action: 'start_game' }));
  
  // Wait for response
  await sleep(2000);
  
  // Clean up
  ws1.close();
  ws2.close();
  
  console.log('\nTest complete');
}

testConnect4().catch(console.error);