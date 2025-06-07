const WebSocket = require('ws');

/**
 * Debug test to capture ALL patches to understand what's happening
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

function connectAndCaptureAll(lobbyId, playerName) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);
    const allPatches = [];
    let messageCount = 0;
    
    ws.on('open', () => {
      console.log(`${playerName} connected`);
    });
    
    ws.on('message', (data) => {
      const msg = JSON.parse(data);
      messageCount++;
      
      console.log(`\n${playerName} Message ${messageCount}:`);
      console.log(`Type: ${msg.type || 'unknown'}`);
      
      if (msg.patches) {
        console.log(`Patches (${msg.patches.length}):`);
        for (let i = 0; i < msg.patches.length; i++) {
          const patch = msg.patches[i];
          console.log(`  ${i + 1}. ${patch.op} ${patch.path}`);
          if (patch.value && typeof patch.value === 'object' && patch.value.entity) {
            console.log(`     Entity: ${patch.value.entity}`);
          } else if (patch.value && typeof patch.value !== 'object') {
            console.log(`     Value: ${JSON.stringify(patch.value)}`);
          }
          allPatches.push(patch);
        }
      } else {
        console.log('No patches in this message');
      }
    });
    
    ws.on('error', reject);
    
    // Keep connection open for 8 seconds to see all dealing
    setTimeout(() => {
      ws.close();
      resolve({ allPatches, messageCount });
    }, 8000);
  });
}

async function testPatchCapture() {
  console.log('🔍 Patch Capture Test\n');
  
  const lobbyId = await createLobby();
  console.log(`Created lobby: ${lobbyId}\n`);
  
  // Connect one player to see patches
  const playerPromise = connectAndCaptureAll(lobbyId, 'Alice');
  
  // Wait a bit then connect second player
  setTimeout(async () => {
    const player2 = await connectAndCaptureAll(lobbyId, 'Bob');
  }, 1000);
  
  // Start the game after both connected
  setTimeout(() => {
    const tempWs = new WebSocket(`${SERVER_URL}/api/lobbies/${lobbyId}/ws?player=starter&join=false`);
    tempWs.on('open', () => {
      console.log('\n🎮 STARTING GAME...\n');
      tempWs.send(JSON.stringify({ action: 'start_game' }));
      tempWs.close();
    });
  }, 2000);
  
  const result = await playerPromise;
  
  console.log(`\n📊 Summary:`);
  console.log(`Total messages: ${result.messageCount}`);
  console.log(`Total patches: ${result.allPatches.length}`);
  
  // Count patches by type
  const patchTypes = {};
  const cardPatches = [];
  
  result.allPatches.forEach(patch => {
    const key = `${patch.op} ${patch.path.split('/')[1] || 'root'}`;
    patchTypes[key] = (patchTypes[key] || 0) + 1;
    
    if (patch.path.includes('zones') && patch.value?.entity) {
      cardPatches.push(`${patch.op} ${patch.path} -> ${patch.value.entity}`);
    }
  });
  
  console.log('\nPatch breakdown:');
  Object.entries(patchTypes).forEach(([type, count]) => {
    console.log(`  ${type}: ${count}`);
  });
  
  if (cardPatches.length > 0) {
    console.log('\nCard-related patches:');
    cardPatches.forEach(patch => console.log(`  ${patch}`));
  } else {
    console.log('\nNo card-related patches found!');
  }
}

testPatchCapture().catch(console.error);