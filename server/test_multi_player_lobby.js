const WebSocket = require('ws');

async function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function testMultiPlayerLobby() {
  // First, create a lobby
  const createRes = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name: 'Multi Player Test' })
  });
  
  const lobby = await createRes.json();
  console.log('Created lobby:', lobby);
  
  // Connect first player
  console.log('\n=== Connecting Player 1 ===');
  const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Player1&join=true`);
  
  ws1.on('open', () => console.log('Player1: Connected'));
  ws1.on('message', (data) => console.log('Player1 received:', JSON.parse(data.toString()).type));
  ws1.on('error', (err) => console.error('Player1 error:', err.message));
  ws1.on('close', () => console.log('Player1: Disconnected'));
  
  await sleep(1000);
  
  // Connect second player
  console.log('\n=== Connecting Player 2 ===');
  const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Player2&join=true`);
  
  ws2.on('open', () => console.log('Player2: Connected'));
  ws2.on('message', (data) => console.log('Player2 received:', JSON.parse(data.toString()).type));
  ws2.on('error', (err) => console.error('Player2 error:', err.message));
  ws2.on('close', () => console.log('Player2: Disconnected'));
  
  await sleep(1000);
  
  // Check lobby state via HTTP
  console.log('\n=== Checking lobby state ===');
  const stateRes = await fetch(`http://localhost:8000/api/lobbies/${lobby.id}`);
  const lobbyState = await stateRes.json();
  console.log('Lobby members:', lobbyState.members);
  
  // Test refreshing (simulating page reload)
  console.log('\n=== Simulating Player1 refresh ===');
  ws1.close();
  await sleep(500);
  
  const ws1Refresh = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Player1&join=false`);
  ws1Refresh.on('open', () => console.log('Player1 (refreshed): Connected'));
  ws1Refresh.on('message', (data) => console.log('Player1 (refreshed) received:', JSON.parse(data.toString()).type));
  ws1Refresh.on('error', (err) => console.error('Player1 (refreshed) error:', err.message));
  
  await sleep(2000);
  
  // Clean up
  ws1Refresh.close();
  ws2.close();
  
  process.exit(0);
}

testMultiPlayerLobby().catch(console.error);
