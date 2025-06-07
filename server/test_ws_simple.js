const WebSocket = require('ws');

async function testSimpleWS() {
    // Create lobby
    const resp = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'go-fish' })
    });
    const lobby = await resp.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect one player
    const ws = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=TestPlayer&join=true`);
    
    ws.on('open', () => {
        console.log('WebSocket connected');
    });
    
    ws.on('message', (data) => {
        console.log('Received:', data.toString());
    });
    
    ws.on('error', (err) => {
        console.error('WebSocket error:', err);
    });
    
    // Wait a bit
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    ws.close();
}

testSimpleWS().catch(console.error);