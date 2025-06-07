const WebSocket = require('ws');

const lobbyId = 'a46d06caad';
const playerName = 'TestPlayer1';

console.log('Connecting to Go Fish lobby:', lobbyId);

const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=${playerName}&join=true`);

ws1.on('open', () => {
  console.log('Player 1 connected');
});

ws1.on('message', (data) => {
  const msg = JSON.parse(data);
  console.log('\n=== Message from server ===');
  console.log('Type:', msg.type);
  
  if (msg.type === 'welcome') {
    console.log('Started:', msg.started);
    console.log('You:', msg.you);
    
    if (msg.ui && msg.ui.zones) {
      console.log('\nZone metadata:');
      msg.ui.zones.forEach(zone => {
        console.log(`  - ${zone.id}: type=${zone.type}, visibility=${zone.visibility}`);
      });
    }
    
    if (msg.game && msg.game.zones) {
      console.log('\nGame zones:');
      Object.keys(msg.game.zones).forEach(zoneId => {
        const zone = msg.game.zones[zoneId];
        console.log(`  - ${zoneId}:`, JSON.stringify(zone).substring(0, 100));
      });
    }
    
    // Add second player
    if (!msg.started) {
      console.log('\nAdding second player...');
      const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=TestPlayer2&join=true`);
      
      ws2.on('open', () => {
        console.log('Player 2 connected');
      });
      
      ws2.on('message', (data2) => {
        const msg2 = JSON.parse(data2);
        if (msg2.type === 'welcome' && !msg2.started) {
          console.log('Player 2 joined, starting game...');
          ws1.send(JSON.stringify({ action: 'start_game' }));
        }
      });
    }
  }
  
  if (msg.type === 'diff' && msg.tick === 1) {
    console.log('\nGame started! Waiting for phase processing...');
  }
  
  if (msg.type === 'diff' && msg.tick === 2) {
    console.log('\nAfter phase processing, checking action map...');
    // Re-connect to get fresh state
    const ws3 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobbyId}/ws?player=TestPlayer1`);
    
    ws3.on('message', (data3) => {
      const msg3 = JSON.parse(data3);
      if (msg3.type === 'welcome' && msg3.started) {
        console.log('\nFinal state check:');
        console.log('Current phase:', msg3.game?.phases?.game);
        
        if (msg3.ui?.actionMap?.p1) {
          console.log('\nAction map for p1:');
          Object.entries(msg3.ui.actionMap.p1).forEach(([location, action]) => {
            console.log(`  ${location}:`, action);
          });
        }
        
        ws1.close();
        ws3.close();
        process.exit(0);
      }
    });
  }
});

ws1.on('error', (err) => {
  console.error('WebSocket error:', err);
});

ws1.on('close', () => {
  console.log('Connection closed');
});