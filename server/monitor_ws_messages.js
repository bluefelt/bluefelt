const WebSocket = require('ws');

async function test() {
    const res = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'tic-tac-toe' })
    });
    const lobby = await res.json();
    console.log('Created lobby:', lobby.id);
    
    const p1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    const p2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    // Track all messages
    const messages = [];
    
    p1.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        messages.push({ from: 'server', to: 'p1', msg });
        console.log(`[P1 <- Server] ${msg.type}`);
        
        if (msg.type === 'error') {
            console.log('  ERROR:', msg.message);
        }
    });
    
    p2.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        messages.push({ from: 'server', to: 'p2', msg });
        console.log(`[P2 <- Server] ${msg.type}`);
    });
    
    // Wait for connections
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Start game
    console.log('\n[P1 -> Server] start_game');
    p1.send(JSON.stringify({ action: 'start_game' }));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Check what phases we're in
    const lastState = messages.filter(m => m.msg.type === 'state' || m.msg.type === 'patch').pop();
    if (lastState && lastState.msg.type === 'patch') {
        const phasePatch = lastState.msg.patches.find(p => p.path === '/game/phases');
        if (phasePatch) {
            console.log('\nCurrent phases:', phasePatch.value);
        }
    }
    
    // Try different message formats
    console.log('\n=== Testing different message formats ===\n');
    
    // Format 1: What the client should send based on useGameActions
    console.log('[P1 -> Server] placeMarker with args.location');
    const msg1 = {
        action: 'placeMarker',
        args: {
            location: '/zones/board/cells/0/0'
        }
    };
    console.log('Sending:', JSON.stringify(msg1));
    p1.send(JSON.stringify(msg1));
    
    await new Promise(resolve => setTimeout(resolve, 500));
    
    // Check if we got any patches
    const recentMessages = messages.slice(-5);
    console.log('\nRecent messages after action:');
    recentMessages.forEach(({ from, to, msg }) => {
        if (msg.type === 'patch') {
            console.log(`${to}: patch with paths:`, msg.patches.map(p => p.path));
        } else {
            console.log(`${to}: ${msg.type}`);
        }
    });
    
    p1.close();
    p2.close();
}

test().catch(console.error);