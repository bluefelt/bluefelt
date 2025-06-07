const WebSocket = require('ws');

async function debugPatches() {
    console.log('Debug patch test...\n');
    
    // Create lobby for tic-tac-toe (simpler game)
    const createResponse = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    
    const lobby = await createResponse.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect one player
    const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    
    ws.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('\n=== MESSAGE RECEIVED ===');
        console.log('Type:', msg.type);
        
        if (msg.type === 'diff') {
            console.log('Tick:', msg.tick);
            console.log('Patch array:', JSON.stringify(msg.patch, null, 2));
        }
        
        if (msg.state) {
            console.log('Initial state received');
            console.log('UI keys:', Object.keys(msg.ui || {}));
            console.log('gameLog initial value:', msg.ui?.gameLog);
        }
    });
    
    // Wait for connection
    await new Promise(resolve => ws.on('open', resolve));
    
    // Add second player
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    await new Promise(resolve => ws2.on('open', resolve));
    
    // Start game
    console.log('\nStarting game...');
    ws.send(JSON.stringify({ action: 'start_game' }));
    
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Make a move - using the correct format
    console.log('\nMaking move...');
    ws.send(JSON.stringify({
        action: 'placeMarker',
        location: '/zones/board/cells/1/1'  // Full path format
    }));
    
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    ws.close();
    ws2.close();
}

debugPatches().catch(console.error);