// Simple integration test for the Lobby → Table → Seat flow
const WebSocket = require('ws');

// Import fetch properly for ESM module
async function loadFetch() {
  if (globalThis.fetch) {
    return globalThis.fetch;
  }
  const module = await import('node-fetch');
  return module.default;
}

async function runTest() {
  const fetch = await loadFetch();
  try {
    console.log('1. Creating lobby...');
    const lobbyResponse = await fetch('http://localhost:8000/api/lobbies', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    if (!lobbyResponse.ok) {
      throw new Error(`Failed to create lobby: ${lobbyResponse.status}`);
    }
    
    const lobby = await lobbyResponse.json();
    console.log('✓ Created lobby:', lobby.id);
    
    console.log('\n2. Connecting WebSocket...');
    const WebSocket = require('ws');
    const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestPlayer&join=true`);
    
    await new Promise((resolve, reject) => {
      ws.on('open', () => {
        console.log('✓ WebSocket connected');
        resolve();
      });
      ws.on('error', reject);
    });
    
    // Wait for lobbyJoined message
    const lobbyJoined = await new Promise((resolve) => {
      ws.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'lobbyJoined') {
          resolve(msg);
        }
      });
    });
    
    console.log('✓ Received lobbyJoined message');
    console.log('  - Lobby name:', lobbyJoined.lobby.name);
    console.log('  - Members:', lobbyJoined.lobby.members.length);
    console.log('  - Tables:', lobbyJoined.lobby.tables?.length || 0);
    
    console.log('\n3. Creating table...');
    ws.send(JSON.stringify({
      action: 'createTable',
      bundleId: 'tic-tac-toe',
      name: 'Test Table'
    }));
    
    // Wait for tableCreated message
    const tableCreated = await new Promise((resolve) => {
      const handler = (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'tableCreated') {
          ws.removeListener('message', handler);
          resolve(msg);
        }
      };
      ws.on('message', handler);
    });
    
    console.log('✓ Table created:', tableCreated.table.id);
    console.log('  - Status:', tableCreated.table.status);
    console.log('  - Seats:', tableCreated.table.seats.length);
    
    console.log('\n4. Claiming seat...');
    ws.send(JSON.stringify({
      action: 'claimSeat',
      tableId: tableCreated.table.id,
      seatIndex: 0
    }));
    
    // Wait for tableUpdated message
    const tableUpdated = await new Promise((resolve) => {
      const handler = (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'tableUpdated') {
          ws.removeListener('message', handler);
          resolve(msg);
        }
      };
      ws.on('message', handler);
    });
    
    console.log('✓ Seat claimed');
    console.log('  - Seat 0:', tableUpdated.seats[0]);
    
    console.log('\n5. Sending chat message...');
    ws.send(JSON.stringify({
      action: 'sendChatMessage',
      message: 'Hello from integration test!',
      scope: 'lobby'
    }));
    
    // Wait for chatMessage
    const chatMessage = await new Promise((resolve) => {
      const handler = (data) => {
        const msg = JSON.parse(data.toString());
        if (msg.type === 'chatMessage') {
          ws.removeListener('message', handler);
          resolve(msg);
        }
      };
      ws.on('message', handler);
    });
    
    console.log('✓ Chat message received');
    console.log('  - Sender:', chatMessage.sender);
    console.log('  - Message:', chatMessage.message);
    
    console.log('\n✅ All integration tests passed!');
    
    ws.close();
    process.exit(0);
    
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    process.exit(1);
  }
}

// Run the test
runTest();