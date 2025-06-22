const WebSocket = require('ws');

async function testTieDetection() {
  const ws1 = new WebSocket('ws://localhost:8000/api/lobbies');
  const ws2 = new WebSocket('ws://localhost:8000/api/lobbies');
  
  let lobbyId;
  let gameState = {};
  
  // Helper to send message
  const send = (ws, msg) => {
    ws.send(JSON.stringify(msg));
  };
  
  // Helper to wait for message
  const waitFor = (ws, type) => new Promise((resolve) => {
    const handler = (data) => {
      const msg = JSON.parse(data);
      if (msg.type === type || (type === 'any' && msg.type)) {
        ws.removeListener('message', handler);
        resolve(msg);
      }
    };
    ws.on('message', handler);
  });
  
  // Create lobby
  await new Promise(resolve => ws1.on('open', resolve));
  send(ws1, { type: 'createLobby', bundleId: 'tic-tac-toe' });
  const createMsg = await waitFor(ws1, 'lobbyCreated');
  lobbyId = createMsg.lobbyId;
  console.log('Created lobby:', lobbyId);
  
  // Join as players
  send(ws1, { type: 'joinLobby', lobbyId, username: 'Alice' });
  await waitFor(ws1, 'lobbyJoined');
  
  await new Promise(resolve => ws2.on('open', resolve));
  send(ws2, { type: 'joinLobby', lobbyId, username: 'Bob' });
  await waitFor(ws2, 'lobbyJoined');
  
  // Start game
  send(ws1, { type: 'startGame' });
  const startMsg = await waitFor(ws1, 'gameStarted');
  console.log('Initial gameStatus:', startMsg.state.gameStatus);
  
  // Play moves that lead to a tie
  const moves = [
    { ws: ws1, pos: '0/0' }, // X
    { ws: ws2, pos: '1/1' }, // O
    { ws: ws1, pos: '0/1' }, // X
    { ws: ws2, pos: '0/2' }, // O
    { ws: ws1, pos: '2/0' }, // X
    { ws: ws2, pos: '1/0' }, // O
    { ws: ws1, pos: '1/2' }, // X
    { ws: ws2, pos: '2/2' }, // O
    { ws: ws1, pos: '2/1' }, // X - final move for tie
  ];
  
  for (let i = 0; i < moves.length; i++) {
    const move = moves[i];
    send(move.ws, {
      type: 'gameAction',
      action: 'placeMark',
      location: `/zones/board/cells/${move.pos}`
    });
    
    const update = await waitFor(move.ws, 'gameUpdate');
    
    // Check gameStatus in patches
    for (const patch of update.patches) {
      if (patch.path === '/game/gameStatus') {
        console.log(`After move ${i+1}, gameStatus:`, patch.value);
        gameState = patch.value;
      }
    }
  }
  
  console.log('Final gameStatus:', gameState);
  console.log('Has tie field:', 'tie' in gameState);
  
  ws1.close();
  ws2.close();
}

testTieDetection().catch(console.error);
