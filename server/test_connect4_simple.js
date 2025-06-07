const WebSocket = require('ws');

async function testConnect4ActionMap() {
  // Create lobby
  const response = await fetch('http://localhost:8000/api/lobbies', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ game_id: 'connect-four' })
  });
  
  const lobby = await response.json();
  console.log('Created lobby:', lobby.id);
  
  // Connect player 1
  const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
  
  ws1.on('message', (data) => {
    const msg = JSON.parse(data);
    
    if (msg.type === 'welcome') {
      console.log('\n=== WELCOME MESSAGE FOR ALICE ===');
      
      // Check lobby state
      if (msg.state?.lobbyState) {
        console.log('\nLobby state keys:', Object.keys(msg.state.lobbyState));
        
        // Check for action map in different locations
        if (msg.state.lobbyState.ui?.actionMap) {
          console.log('\nFound action map at lobbyState.ui.actionMap');
          console.log('Action map:', JSON.stringify(msg.state.lobbyState.ui.actionMap, null, 2));
        } else if (msg.state.lobbyState.actionMap) {
          console.log('\nFound action map at lobbyState.actionMap');
          console.log('Action map:', JSON.stringify(msg.state.lobbyState.actionMap, null, 2));
        } else {
          console.log('\nNo action map found. Full lobbyState structure:');
          console.log(JSON.stringify(msg.state.lobbyState, null, 2));
        }
      } else {
        console.log('\nNo lobbyState found. Full message:');
        console.log(JSON.stringify(msg, null, 2));
      }
      
      ws1.close();
      process.exit(0);
    }
  });
  
  ws1.on('error', (err) => {
    console.error('WebSocket error:', err);
    process.exit(1);
  });
}

testConnect4ActionMap().catch(console.error);