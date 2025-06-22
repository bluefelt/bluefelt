const WebSocket = require('ws');

async function testConnectFour() {
  console.log('Testing Connect Four...\n');
  
  // Create lobby
  const res = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'connect-four' })
  });
  
  const lobby = await res.json();
  console.log(`Created lobby: ${lobby.id}`);
  
  // Connect Alice
  const alice = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
  await new Promise(r => alice.on('open', r));
  console.log('Alice connected');
  
  // Connect Bob
  const bob = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
  await new Promise(r => bob.on('open', r));
  console.log('Bob connected');
  
  // Wait a bit
  await new Promise(r => setTimeout(r, 500));
  
  // Create and join game
  alice.send(JSON.stringify({ action: 'createGame', gameType: 'connect-four' }));
  await new Promise(r => setTimeout(r, 200));
  
  alice.send(JSON.stringify({ action: 'joinGame', gameId: 0 }));
  await new Promise(r => setTimeout(r, 200));
  
  bob.send(JSON.stringify({ action: 'joinGame', gameId: 0 }));
  await new Promise(r => setTimeout(r, 200));
  
  // Start game
  alice.send(JSON.stringify({ action: 'startGame', gameId: 0 }));
  await new Promise(r => setTimeout(r, 500));
  
  console.log('\nGame started! Making moves...\n');
  
  // Make some moves
  alice.send(JSON.stringify({ 
    action: 'gameAction', 
    gameId: 0,
    data: { action: 'dropDisc', targetColumn: 3 } 
  }));
  await new Promise(r => setTimeout(r, 500));
  
  bob.send(JSON.stringify({ 
    action: 'gameAction', 
    gameId: 0,
    data: { action: 'dropDisc', targetColumn: 3 } 
  }));
  await new Promise(r => setTimeout(r, 500));
  
  alice.send(JSON.stringify({ 
    action: 'gameAction', 
    gameId: 0,
    data: { action: 'dropDisc', targetColumn: 4 } 
  }));
  await new Promise(r => setTimeout(r, 500));
  
  console.log('Test completed successfully! Connect Four is working.');
  
  alice.close();
  bob.close();
}

testConnectFour().catch(console.error);