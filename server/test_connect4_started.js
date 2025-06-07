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
  
  return new Promise((resolve, reject) => {
    // Connect player 1
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    
    // Connect player 2
    let ws2;
    
    ws1.on('open', () => {
      console.log('Alice connected');
      
      // Connect player 2 after player 1 is connected
      ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
      
      ws2.on('open', () => {
        console.log('Bob connected');
        
        // Start the game
        setTimeout(() => {
          console.log('Starting game...');
          ws1.send(JSON.stringify({ action: 'start_game' }));
        }, 500);
      });
    });
    
    ws1.on('message', (data) => {
      const msg = JSON.parse(data);
      
      if (msg.type === 'patch') {
        // Look for action map in patches
        const actionMapPatch = msg.patch?.find(p => p.path === '/ui/actionMap');
        if (actionMapPatch) {
          console.log('\n=== ACTION MAP FROM PATCH ===');
          console.log(JSON.stringify(actionMapPatch.value, null, 2));
          
          console.log('\n=== ACTION MAP KEYS ===');
          Object.keys(actionMapPatch.value).forEach(key => {
            console.log(`- ${key}: ${actionMapPatch.value[key].action}`);
          });
          
          // Look for a specific column to understand the format
          console.log('\n=== SAMPLE COLUMN ACTIONS ===');
          Object.entries(actionMapPatch.value).forEach(([key, value]) => {
            if (key.includes('board') && value.action === 'drop') {
              console.log(`${key}: ${JSON.stringify(value)}`);
            }
          });
          
          ws1.close();
          if (ws2) ws2.close();
          resolve();
        }
      }
    });
    
    ws1.on('error', reject);
  });
}

testConnect4ActionMap()
  .then(() => {
    console.log('\nTest complete');
    process.exit(0);
  })
  .catch((err) => {
    console.error('Test failed:', err);
    process.exit(1);
  });