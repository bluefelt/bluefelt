const WebSocket = require('ws');

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testGameJoinIssue() {
  console.log('Creating lobby...');
  // Create a lobby
  const createRes = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: 'Test Game Join' })
  });
  
  const lobby = await createRes.json();
  console.log('Created lobby:', lobby.id);
  
  // Connect P1
  console.log('\nConnecting P1...');
  const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Player1&join=true`);
  
  await new Promise((resolve, reject) => {
    ws1.on('open', resolve);
    ws1.on('error', reject);
  });
  
  ws1.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log('P1 received:', msg.type);
  });
  
  await sleep(500);
  
  // Connect P2
  console.log('\nConnecting P2...');
  const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Player2&join=true`);
  
  await new Promise((resolve, reject) => {
    ws2.on('open', resolve);
    ws2.on('error', reject);
  });
  
  ws2.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log('P2 received:', msg.type);
  });
  
  await sleep(500);
  
  // P1 creates a game
  console.log('\nP1 creating game...');
  ws1.send(JSON.stringify({
    action: 'create_game',
    game_type: 'tic-tac-toe'
  }));
  
  await sleep(500);
  
  // P2 joins the game
  console.log('\nP2 attempting to join game...');
  // First get lobby state to find game ID
  const lobbyRes = await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`);
  const lobbyState = await lobbyRes.json();
  console.log('Current games:', lobbyState.games);
  
  if (lobbyState.games && lobbyState.games.length > 0) {
    const gameId = lobbyState.games[0].id;
    console.log('P2 joining game:', gameId);
    
    ws2.send(JSON.stringify({
      action: 'join_game',
      game_id: gameId
    }));
    
    await sleep(500);
  }
  
  // Test if server is still responsive
  console.log('\nTesting server responsiveness...');
  try {
    const healthRes = await fetch('http://localhost:8000/health');
    console.log('Health check:', healthRes.ok ? 'OK' : 'FAILED');
    
    const lobbiesRes = await fetch('http://localhost:8000/api/lobbies');
    console.log('Lobbies endpoint:', lobbiesRes.ok ? 'OK' : 'FAILED');
  } catch (err) {
    console.error('Server not responding:', err.message);
  }
  
  // Clean up
  ws1.close();
  ws2.close();
}

testGameJoinIssue().catch(console.error);
