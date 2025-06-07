const WebSocket = require('ws');

const SERVER_URL = 'http://localhost:8000';
const WS_URL = 'ws://localhost:8000';

async function createLobby(gameId) {
  const response = await fetch(`${SERVER_URL}/api/lobbies`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: gameId })
  });
  
  if (!response.ok) {
    throw new Error(`Failed to create lobby: ${response.status}`);
  }
  
  return await response.json();
}

function connectPlayer(lobbyId, playerName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${WS_URL}/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
    
    ws.on('open', () => {
      console.log(`✓ ${playerName} connected`);
    });
    
    let resolved = false;
    
    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data);
        console.log(`\n${playerName} received message type: ${msg.type}`);
        
        if (msg.type === 'welcome') {
          console.log(`\n${playerName} received welcome message`);
          
          // Log the full action map
          if (msg.state?.ui?.actionMap) {
            console.log(`\n=== ACTION MAP FOR ${playerName} ===`);
            console.log(JSON.stringify(msg.state.ui.actionMap, null, 2));
            
            // Also log just the keys to see the format
            console.log(`\n=== ACTION MAP KEYS ===`);
            Object.keys(msg.state.ui.actionMap).forEach(key => {
              console.log(`- ${key}`);
            });
          } else {
            console.log(`No action map found in welcome message for ${playerName}`);
            console.log('Welcome message structure:', JSON.stringify(msg, null, 2));
          }
          
          if (!resolved) {
            resolved = true;
            resolve({ ws, welcomeMsg: msg });
          }
        }
      } catch (err) {
        console.error('Error parsing message:', err);
      }
    });
    
    ws.on('error', reject);
    ws.on('close', () => {
      console.log(`${playerName} disconnected`);
    });
  });
}

async function startGame(ws) {
  return new Promise((resolve) => {
    ws.send(JSON.stringify({ action: 'start_game' }));
    
    ws.on('message', (data) => {
      try {
        const msg = JSON.parse(data);
        if (msg.type === 'patch' && msg.patch?.some(p => p.path === '/game/phase' && p.value === 'playing')) {
          console.log('\n✓ Game started successfully');
          
          // Check if there's an updated action map in the patch
          const actionMapPatch = msg.patch?.find(p => p.path === '/ui/actionMap');
          if (actionMapPatch) {
            console.log('\n=== UPDATED ACTION MAP AFTER GAME START ===');
            console.log(JSON.stringify(actionMapPatch.value, null, 2));
            
            console.log('\n=== UPDATED ACTION MAP KEYS ===');
            Object.keys(actionMapPatch.value).forEach(key => {
              console.log(`- ${key}`);
            });
          }
          
          resolve();
        }
      } catch (err) {
        // Ignore parse errors
      }
    });
  });
}

async function main() {
  console.log('Testing Connect 4 Action Map Format...\n');
  
  try {
    // Create lobby
    console.log('1. Creating Connect 4 lobby...');
    const lobby = await createLobby('connect-four');
    console.log(`✓ Lobby created: ${lobby.id}`);
    
    // Connect players
    console.log('\n2. Connecting players...');
    const player1 = await connectPlayer(lobby.id, 'Alice');
    const player2 = await connectPlayer(lobby.id, 'Bob');
    
    // Start game
    console.log('\n3. Starting game...');
    await startGame(player1.ws);
    
    // Wait a bit to see if any additional messages come through
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Clean up
    player1.ws.close();
    player2.ws.close();
    
    console.log('\n✓ Test complete');
    
  } catch (error) {
    console.error('Test failed:', error);
  }
}

// Run the test
main().catch(console.error);