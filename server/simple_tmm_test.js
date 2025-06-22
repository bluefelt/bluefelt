const WebSocket = require('ws');

async function testThreeMensMorris() {
    console.log('=== Simple Three Men\'s Morris Test ===\n');
    
    // Create lobby
    const response = await fetch('http://localhost:8000/api/lobbies', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ game_id: 'three-mens-morris' })
    });
    const lobby = await response.json();
    console.log('Created lobby:', lobby.id);
    
    // Connect player 1
    const ws1 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Alice&join=true`);
    
    await new Promise(resolve => ws1.on('open', resolve));
    console.log('Alice connected');
    
    // Connect player 2  
    const ws2 = new WebSocket(`ws://localhost:8000/api/lobbies/${lobby.id}/ws?player=Bob&join=true`);
    
    await new Promise(resolve => ws2.on('open', resolve));
    console.log('Bob connected');
    
    // Log all messages
    ws1.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('Alice received:', msg.type);
        if (msg.type === 'gameStarted') {
            console.log('  Phase:', msg.game?.phases?.game);
            console.log('  Current player:', msg.game?.currentPlayer);
            if (msg.ui?.actionMap?.p1) {
                const actions = Object.keys(msg.ui.actionMap.p1);
                console.log('  Alice action count:', actions.length);
                if (actions.length > 0) {
                    console.log('  Sample action:', actions[0]);
                    console.log('  Action info:', msg.ui.actionMap.p1[actions[0]]);
                }
            }
        }
        if (msg.type === 'diff') {
            console.log('  Patches received:', msg.patches?.length || 0);
            if (msg.patches) {
                msg.patches.forEach(patch => {
                    if (patch.path.includes('actionMap') || patch.path.includes('phase')) {
                        console.log('    Patch:', patch.op, patch.path);
                    }
                });
            }
        }
    });
    
    ws2.on('message', (data) => {
        const msg = JSON.parse(data);
        console.log('Bob received:', msg.type);
    });
    
    // Wait for initial setup
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Start game
    console.log('\nStarting game...');
    ws1.send(JSON.stringify({ action: 'start_game' }));
    
    // Wait for game to start
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    // Try to place a piece
    console.log('\nTrying to place piece...');
    ws1.send(JSON.stringify({
        action: 'placeToken',
        args: { target: '/zones/board/cells/0/0' }
    }));
    
    // Wait for placement
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Close connections
    ws1.close();
    ws2.close();
}

testThreeMensMorris().catch(console.error);