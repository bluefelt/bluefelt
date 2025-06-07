const WebSocket = require('ws');

/**
 * Simple debug test to see exactly what messages are being sent during game start
 */

const SERVER_URL = 'ws://localhost:8000';
const API_URL = 'http://localhost:8000';

async function createLobby() {
  const response = await fetch(`${API_URL}/api/lobbies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'go-fish' })
  });
  const data = await response.json();
  return data.id;
}

function connectPlayer(lobbyId, playerName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
    const messages = [];
    
    ws.on('open', () => {
      console.log(`\n${playerName} connected`);
    });
    
    ws.on('message', (data) => {
      const msg = JSON.parse(data);
      messages.push(msg);
      
      console.log(`\n${playerName} received message:`);
      console.log(JSON.stringify(msg, null, 2));
      
      // Log specifically about card movements
      if (msg.patches) {
        for (const patch of msg.patches) {
          if (patch.path.includes('/zones/') && (patch.path.includes('hand_') || patch.path.includes('pool'))) {
            console.log(`  🃏 Card-related patch: ${patch.op} at ${patch.path}`);
            if (patch.value && patch.value.entity) {
              console.log(`    Entity: ${patch.value.entity}`);
            }
          }
          if (patch.path === '/game/phases/game') {
            console.log(`  📋 Phase change: ${patch.value}`);
          }
        }
      }
      
      // Look for pool zone contents in welcome messages
      if (msg.lobbyState && msg.lobbyState.zones && msg.lobbyState.zones.pool) {
        console.log(`\n  🃏 Pool zone has ${msg.lobbyState.zones.pool.items.length} items:`);
        const first5 = msg.lobbyState.zones.pool.items.slice(0, 5);
        const last5 = msg.lobbyState.zones.pool.items.slice(-5);
        console.log(`    First 5: ${first5.map(i => i.entity).join(', ')}`);
        console.log(`    Last 5: ${last5.map(i => i.entity).join(', ')}`);
      }
    });
    
    ws.on('error', reject);
    
    resolve({ ws, messages, playerName });
  });
}

async function debugTest() {
  console.log('🔍 Debug Test: Go Fish Message Flow');
  
  const lobbyId = await createLobby();
  console.log(`\nCreated lobby: ${lobbyId}`);
  
  // Connect players
  const player1 = await connectPlayer(lobbyId, 'Alice');
  await new Promise(resolve => setTimeout(resolve, 500));
  const player2 = await connectPlayer(lobbyId, 'Bob');
  
  await new Promise(resolve => setTimeout(resolve, 1000));
  
  // Start the game
  console.log('\n🎮 Starting game...');
  player1.ws.send(JSON.stringify({ action: 'start_game' }));
  
  // Wait and see what happens
  await new Promise(resolve => setTimeout(resolve, 5000));
  
  console.log('\n📊 Summary:');
  console.log(`Player 1 received ${player1.messages.length} messages`);
  console.log(`Player 2 received ${player2.messages.length} messages`);
  
  // Clean up
  player1.ws.close();
  player2.ws.close();
}

debugTest().catch(console.error);