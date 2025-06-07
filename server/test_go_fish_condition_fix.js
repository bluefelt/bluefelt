#!/usr/bin/env node

// Test the Go Fish condition fix 

const WebSocket = require('ws');

async function testConditionFix() {
  // Create lobby
  const createResponse = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  
  const lobby = await createResponse.json();
  console.log('✓ Created lobby:', lobby.id);
  
  let actionCount = 0;
  let gameProgressed = false;
  
  return new Promise((resolve) => {
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestPlayer1&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestPlayer2&join=true`);
    
    ws1.on('message', (data) => {
      const message = JSON.parse(data);
      
      if (message.type === 'welcome') {
        console.log('✓ Player 1 connected');
      }
      
      if (message.type === 'stateUpdate') {
        actionCount++;
        console.log('✓ State update', actionCount, '- patches:', message.patches?.length || 0);
        
        // Check for game log messages indicating progression
        message.patches?.forEach(patch => {
          if (patch.path === '/ui/gameLog/-' && patch.value?.message) {
            console.log('  Log:', patch.value.message);
            gameProgressed = true;
          }
        });
      }
    });
    
    ws2.on('message', (data) => {
      const message = JSON.parse(data);
      if (message.type === 'welcome') {
        console.log('✓ Player 2 connected');
        // Start game
        setTimeout(() => {
          console.log('Starting game...');
          ws1.send(JSON.stringify({ action: 'start_game' }));
        }, 500);
      }
    });
    
    // Test completion
    setTimeout(() => {
      console.log('\n=== Test Results ===');
      console.log('Total state updates:', actionCount);
      console.log('Game progressed beyond initial setup:', gameProgressed);
      
      if (actionCount > 5 && gameProgressed) {
        console.log('✅ CONDITION FIX SUCCESSFUL - Game is progressing normally!');
      } else {
        console.log('❌ Game may still be having issues');
      }
      
      ws1.close();
      ws2.close();
      resolve();
    }, 8000);
  });
}

testConditionFix().then(() => {
  console.log('Test completed');
  process.exit(0);
}).catch(console.error);