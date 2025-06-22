const WebSocket = require('ws');
const http = require('http');

async function createLobby() {
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
          resolve(lobby);
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

async function runTest() {
  try {
    const lobby = await createLobby();
    console.log('Created lobby:', lobby.id);
    
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=p1&join=true`);
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=p2&join=true`);
    
    let playersJoined = 0;
    let gameStarted = false;
    
    const waitForConnection = () => new Promise(resolve => {
      ws1.on('open', () => {
        playersJoined++;
        if (playersJoined === 2) resolve();
      });
      ws2.on('open', () => {
        playersJoined++;
        if (playersJoined === 2) resolve();
      });
    });
    
    await waitForConnection();
    console.log('Both players connected');
    
    ws1.on('message', (data) => {
      const msg = JSON.parse(data);
      
      if (msg.type === 'lobby_update' && msg.data.players && msg.data.players.length === 2 && !gameStarted) {
        gameStarted = true;
        console.log('Starting game...');
        ws1.send(JSON.stringify({action: 'start_game'}));
      }
      
      if (msg.type === 'game_state') {
        console.log('\n=== GAME STATE ===');
        console.log('Current player:', msg.data.currentPlayer);
        console.log('Game ended:', msg.data.gameStatus ? msg.data.gameStatus.state === 'ended' : 'no status');
        
        if (msg.data.zones && msg.data.zones.board) {
          const cells = msg.data.zones.board.cells;
          console.log('Board:');
          for (let r = 0; r < cells.length; r++) {
            let row = '';
            for (let c = 0; c < cells[r].length; c++) {
              const cell = cells[r][c];
              if (cell && cell.entity) {
                if (cell.entity === 'mark_p1') row += 'X ';
                else if (cell.entity === 'mark_p2') row += 'O ';
                else row += '? ';
              } else {
                row += '- ';
              }
            }
            console.log('  ' + row);
          }
        }
        
        const p1Actions = msg.data.ui && msg.data.ui.actionMap ? Object.keys(msg.data.ui.actionMap.p1 || {}) : [];
        const p2Actions = msg.data.ui && msg.data.ui.actionMap ? Object.keys(msg.data.ui.actionMap.p2 || {}) : [];
        
        console.log('P1 actions:', p1Actions.length);
        console.log('P2 actions:', p2Actions.length);
        
        // If current player has actions, make a move
        if (msg.data.currentPlayer === 'p1' && p1Actions.length > 0 && !msg.data.gameStatus) {
          const validCell = p1Actions.find(action => action.includes('/zones/board/cells/'));
          if (validCell) {
            console.log('P1 moving to:', validCell);
            ws1.send(JSON.stringify({ action: 'placeMark', args: { location: validCell } }));
          }
        } else if (msg.data.currentPlayer === 'p2' && p2Actions.length > 0 && !msg.data.gameStatus) {
          const validCell = p2Actions.find(action => action.includes('/zones/board/cells/'));
          if (validCell) {
            console.log('P2 moving to:', validCell);
            ws2.send(JSON.stringify({ action: 'placeMark', args: { location: validCell } }));
          }
        }
      }
    });
    
    ws2.on('message', () => {}); // Just to keep connection alive
    
    setTimeout(() => {
      ws1.close();
      ws2.close();
      process.exit(0);
    }, 15000);
    
  } catch (error) {
    console.error('Test failed:', error);
    process.exit(1);
  }
}

runTest();