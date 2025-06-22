const WebSocket = require('ws');
const http = require('http');

// Simple test to create a lobby and send some actions
function createLobby() {
  return new Promise((resolve, reject) => {
    const options = {
      hostname: 'localhost',
      port: 8000,
      path: '/api/lobbies',
      method: 'POST',
      headers: { 'Content-Type': 'application/json' }
    };

    const req = http.request(options, (res) => {
      let data = '';
      res.on('data', (chunk) => data += chunk);
      res.on('end', () => {
        try {
          const lobby = JSON.parse(data);
          resolve(lobby.id);
        } catch (e) {
          reject(e);
        }
      });
    });

    req.on('error', reject);
    req.write(JSON.stringify({game_id: 'tic-tac-toe'}));
    req.end();
  });
}

async function test() {
  try {
    const lobbyId = await createLobby();
    console.log('Lobby created:', lobbyId);
    
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=p1&join=true`);
    
    ws1.on('open', () => {
      console.log('P1 connected');
      
      // Try starting the game directly
      setTimeout(() => {
        console.log('Sending start_game');
        ws1.send(JSON.stringify({action: 'start_game'}));
      }, 1000);
      
      setTimeout(() => {
        console.log('Closing connection');
        ws1.close();
      }, 3000);
    });
    
    ws1.on('message', (data) => {
      const msg = JSON.parse(data);
      console.log('Message type:', msg.type);
      
      if (msg.type === 'game_state') {
        console.log('Game state received');
        console.log('Current player:', msg.data.currentPlayer);
        
        // Make a move if it's our turn
        if (msg.data.currentPlayer === 'p1' && msg.data.ui && msg.data.ui.actionMap && msg.data.ui.actionMap.p1) {
          const actions = Object.keys(msg.data.ui.actionMap.p1);
          console.log('P1 actions available:', actions.length);
          
          const moveAction = actions.find(a => a.includes('/zones/board/cells/'));
          if (moveAction) {
            console.log('Making move to:', moveAction);
            ws1.send(JSON.stringify({ action: 'placeMark', args: { location: moveAction } }));
          }
        }
      }
    });
    
    ws1.on('error', (err) => {
      console.log('WebSocket error:', err.message);
    });
    
    ws1.on('close', () => {
      console.log('WebSocket closed');
      process.exit(0);
    });
    
  } catch (error) {
    console.error('Error:', error);
    process.exit(1);
  }
}

test();